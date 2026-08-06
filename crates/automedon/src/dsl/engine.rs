use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rhai::{Dynamic, Engine, Map, Scope};
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::config::LaunchOptions;
use crate::error::{Error, Result};
use crate::expect::Expect;
use crate::session::Session;
use crate::wait::Wait;

type RhaiResult<T> = std::result::Result<T, Box<rhai::EvalAltResult>>;

/// Outcome of running a Rhai harness script.
#[derive(Debug)]
pub struct ScriptResult {
    pub value: Dynamic,
}

/// Shared session handle inside Rhai (sync API over tokio Handle).
#[derive(Clone)]
struct SessionHandle {
    inner: Arc<Mutex<Session>>,
    rt: Handle,
}

impl SessionHandle {
    fn block<T>(&self, fut: impl std::future::Future<Output = T>) -> T {
        self.rt.block_on(fut)
    }
}

/// Evaluate a Rhai script string.
pub fn eval_str(source: &str) -> Result<ScriptResult> {
    match Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| eval_with_handle(source, handle)),
        Err(_) => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| Error::Other(e.to_string()))?;
            eval_with_handle(source, rt.handle().clone())
        }
    }
}

/// Evaluate a `.rhai` / `.ail` file.
pub fn eval_file(path: impl AsRef<Path>) -> Result<ScriptResult> {
    let source = std::fs::read_to_string(path.as_ref())?;
    eval_str(&source)
}

/// Alias used by the CLI.
pub fn run_script(path: impl AsRef<Path>) -> Result<ScriptResult> {
    eval_file(path)
}

fn eval_with_handle(source: &str, rt: Handle) -> Result<ScriptResult> {
    let engine = make_engine(rt);
    let mut scope = Scope::new();
    let value = engine
        .eval_with_scope::<Dynamic>(&mut scope, source)
        .map_err(|e| Error::Script(e.to_string()))?;
    Ok(ScriptResult { value })
}

fn make_engine(rt: Handle) -> Engine {
    let mut engine = Engine::new();
    engine.set_max_expr_depths(128, 128);
    engine.set_max_operations(1_000_000);

    engine
        .register_type_with_name::<Expect>("Expect")
        .register_type_with_name::<Wait>("Wait")
        .register_type_with_name::<SessionHandle>("Session");

    // Expect constructors (still supported)
    engine.register_fn("text", |s: &str| Expect::text(s));
    engine.register_fn("thinking", |s: &str| Expect::thinking(s));
    engine.register_fn("tool", |s: &str| Expect::tool(s));
    engine.register_fn("tool_any", Expect::tool_any);
    engine.register_fn("tool_result", |s: &str| Expect::tool_result(s));
    engine.register_fn("done", Expect::done);
    engine.register_fn("turn_complete", Expect::turn_complete);
    engine.register_fn("process_exit", Expect::process_exit);
    engine.register_fn("permission", Expect::permission);
    engine.register_fn("hook", |s: &str| Expect::hook(s));
    engine.register_fn("hook_any", Expect::hook_any);
    engine.register_fn("hook_started", |s: &str| Expect::hook_started(s));
    engine.register_fn("hook_finished", |s: &str| Expect::hook_finished(s));
    engine.register_fn("hook_phase", |name: &str, phase: &str| {
        Expect::hook_phase(name, phase)
    });
    engine.register_fn("plan", Expect::plan);
    engine.register_fn("plan_summary", |s: &str| Expect::plan_summary(s));
    engine.register_fn("plan_resolved", |approved: bool| {
        Expect::plan_resolved(approved)
    });
    engine.register_fn("goal", Expect::goal);
    engine.register_fn("goal_title", |s: &str| Expect::goal_title(s));
    engine.register_fn("goal_progress", Expect::goal_progress);
    engine.register_fn("goal_completed", |ok: bool| Expect::goal_completed(ok));
    engine.register_fn("session_info", Expect::session_info);
    engine.register_fn("timeout_ms", |exp: Expect, ms: i64| {
        exp.timeout(Duration::from_millis(ms as u64))
    });

    // Wait constructors (preferred for new scripts)
    engine.register_fn("wait_text", |s: &str| Wait::text(s));
    engine.register_fn("wait_tool", |s: &str| Wait::tool(s));
    engine.register_fn("wait_tool_any", Wait::tool_any);
    engine.register_fn("wait_tool_result", |s: &str| Wait::tool_result(s));
    engine.register_fn("wait_permission", Wait::permission);
    engine.register_fn("wait_hook", |s: &str| Wait::hook(s));
    engine.register_fn("wait_hook_any", Wait::hook_any);
    engine.register_fn("wait_hook_started", |s: &str| Wait::hook_started(s));
    engine.register_fn("wait_hook_finished", |s: &str| Wait::hook_finished(s));
    engine.register_fn("wait_hook_phase", |name: &str, phase: &str| {
        Wait::hook_phase(name, phase)
    });
    engine.register_fn("wait_plan", Wait::plan);
    engine.register_fn("wait_goal", Wait::goal);
    engine.register_fn("wait_turn_complete", Wait::turn_complete);
    engine.register_fn("wait_done", Wait::done);
    engine.register_fn("wait_timeout_ms", |w: Wait, ms: i64| {
        w.timeout(Duration::from_millis(ms as u64))
    });
    // timeout_ms also accepts Wait (overload)
    engine.register_fn("timeout_ms", |w: Wait, ms: i64| {
        w.timeout(Duration::from_millis(ms as u64))
    });

    let rt_launch = rt.clone();
    engine.register_fn("launch", move |name: &str| -> RhaiResult<SessionHandle> {
        launch_inner(name, Map::new(), rt_launch.clone())
    });

    let rt_launch2 = rt.clone();
    engine.register_fn(
        "launch",
        move |name: &str, opts: Map| -> RhaiResult<SessionHandle> {
            launch_inner(name, opts, rt_launch2.clone())
        },
    );

    engine.register_fn(
        "prompt",
        |s: &mut SessionHandle, text: &str| -> RhaiResult<()> {
            let text = text.to_string();
            let inner = s.inner.clone();
            s.block(async move { inner.lock().await.prompt(&text).await.map_err(to_rhai_err) })
        },
    );

    engine.register_fn("await_turn", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.await_turn().await.map_err(to_rhai_err) })
    });

    engine.register_fn(
        "run",
        |s: &mut SessionHandle, text: &str| -> RhaiResult<String> {
            let text = text.to_string();
            let inner = s.inner.clone();
            s.block(async move {
                let r = inner.lock().await.run(&text).await.map_err(to_rhai_err)?;
                Ok(r.turn_text)
            })
        },
    );

    engine.register_fn(
        "expect",
        |s: &mut SessionHandle, exp: Expect| -> RhaiResult<()> {
            let inner = s.inner.clone();
            s.block(async move {
                inner.lock().await.expect(exp).await.map_err(to_rhai_err)?;
                Ok(())
            })
        },
    );

    engine.register_fn("wait", |s: &mut SessionHandle, w: Wait| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move {
            inner.lock().await.wait(w).await.map_err(to_rhai_err)?;
            Ok(())
        })
    });

    engine.register_fn(
        "wait_for",
        |s: &mut SessionHandle, w: Wait| -> RhaiResult<()> {
            let inner = s.inner.clone();
            s.block(async move {
                inner.lock().await.wait_for(w).await.map_err(to_rhai_err)?;
                Ok(())
            })
        },
    );

    engine.register_fn("approve", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.approve().await.map_err(to_rhai_err) })
    });

    engine.register_fn("deny", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.deny().await.map_err(to_rhai_err) })
    });

    engine.register_fn("approve_plan", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.approve_plan().await.map_err(to_rhai_err) })
    });

    engine.register_fn("reject_plan", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.reject_plan().await.map_err(to_rhai_err) })
    });

    engine.register_fn("drain", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move {
            inner
                .lock()
                .await
                .drain_until_done()
                .await
                .map_err(to_rhai_err)
        })
    });

    engine.register_fn("close", |s: &mut SessionHandle| -> RhaiResult<()> {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.close().await.map_err(to_rhai_err) })
    });

    engine.register_fn("text", |s: &mut SessionHandle| -> String {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.text().to_string() })
    });

    engine.register_fn("turn_text", |s: &mut SessionHandle| -> String {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.turn_text().to_string() })
    });

    engine.register_fn("thinking", |s: &mut SessionHandle| -> String {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.thinking().to_string() })
    });

    engine.register_fn("session_id", |s: &mut SessionHandle| -> String {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.session_id().unwrap_or("").to_string() })
    });

    engine.register_fn("harness", |s: &mut SessionHandle| -> String {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.harness().to_string() })
    });

    engine.register_fn("finished", |s: &mut SessionHandle| -> bool {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.is_finished() })
    });

    engine.register_fn("turn", |s: &mut SessionHandle| -> i64 {
        let inner = s.inner.clone();
        s.block(async move { inner.lock().await.turn() as i64 })
    });

    engine.register_fn("tool_names", |s: &mut SessionHandle| -> Dynamic {
        let inner = s.inner.clone();
        s.block(async move {
            let names: Vec<Dynamic> = inner
                .lock()
                .await
                .transcript()
                .tools()
                .iter()
                .map(|t| Dynamic::from(t.name.clone()))
                .collect();
            Dynamic::from_array(names)
        })
    });

    engine.register_fn(
        "assert_contains",
        |hay: &str, needle: &str| -> RhaiResult<()> {
            if hay.contains(needle) {
                Ok(())
            } else {
                Err(format!("assert_contains failed: {needle:?} not in {hay:?}").into())
            }
        },
    );

    engine.register_fn("assert_true", |v: bool| -> RhaiResult<()> {
        if v {
            Ok(())
        } else {
            Err("assert_true failed".into())
        }
    });

    engine
}

fn launch_inner(name: &str, opts_map: Map, rt: Handle) -> RhaiResult<SessionHandle> {
    let opts = map_to_launch_options(opts_map);
    let session = Session::builder(name)
        .opts(opts)
        .build()
        .map_err(to_rhai_err)?;
    Ok(SessionHandle {
        inner: Arc::new(Mutex::new(session)),
        rt,
    })
}

fn map_to_launch_options(map: Map) -> LaunchOptions {
    let mut opts = LaunchOptions::default();
    for (k, v) in map.iter() {
        let key = k.as_str();
        match key {
            "cwd" => {
                if let Ok(s) = v.clone().into_string() {
                    opts.cwd = Some(s.into());
                }
            }
            "bin" => {
                if let Ok(s) = v.clone().into_string() {
                    opts.bin = Some(s.into());
                }
            }
            "model" => {
                if let Ok(s) = v.clone().into_string() {
                    opts.model = Some(s);
                }
            }
            "yolo" => {
                if let Ok(b) = v.as_bool() {
                    opts.yolo = b;
                }
            }
            "timeout_ms" => {
                if let Ok(n) = v.as_int() {
                    opts.default_timeout = Some(Duration::from_millis(n as u64));
                }
            }
            other => {
                opts.extra.insert(other.to_string(), dynamic_to_json(v));
            }
        }
    }
    opts
}

fn dynamic_to_json(v: &Dynamic) -> Value {
    if let Ok(s) = v.clone().into_string() {
        return Value::String(s);
    }
    if let Ok(b) = v.as_bool() {
        return Value::Bool(b);
    }
    if let Ok(i) = v.as_int() {
        return Value::Number(i.into());
    }
    if let Ok(f) = v.as_float() {
        return serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null);
    }
    if let Some(m) = v.clone().try_cast::<Map>() {
        let mut obj = serde_json::Map::new();
        for (k, val) in m.iter() {
            obj.insert(k.to_string(), dynamic_to_json(val));
        }
        return Value::Object(obj);
    }
    Value::String(v.to_string())
}

fn to_rhai_err(e: Error) -> Box<rhai::EvalAltResult> {
    e.to_string().into()
}
