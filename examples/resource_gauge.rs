//! resource_gauge example -- K8s pod-quota shape.
//!
//! Demonstrates the ResourceGauge component with a realistic Kubernetes
//! pod-quota scenario. Each pod's CPU (millicores) and memory (MB)
//! consumption is shown against its request and limit, using the
//! post-closure builder surface introduced in this cadence:
//!
//! - `ResourceGaugeState::default().with_values(ResourceValues { .. })`
//!   replaces the removed positional `ResourceGaugeState::new(a, r, l)`.
//! - `state.values()` returns the named `ResourceValues` struct back out,
//!   matching the existing `set_values` mutator.
//!
//! Navigate with Up/Down (or j/k) to move the highlighted-pod border; q/Esc
//! quits.
//!
//! Run with: cargo run --example resource_gauge --features display-components

use envision::prelude::*;

/// A pod's resource snapshot at one point in time.
#[derive(Clone, Debug)]
struct Pod {
    name: &'static str,
    cpu: ResourceValues,
    memory: ResourceValues,
}

/// Application marker type.
struct QuotaApp;

/// Application state: a fixed roster of pods plus which row is highlighted.
#[derive(Clone)]
struct State {
    pods: Vec<Pod>,
    selected: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Next,
    Prev,
    Quit,
}

impl App for QuotaApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let pods = vec![
            Pod {
                name: "api-server-x7fk2",
                cpu: ResourceValues {
                    actual: 350.0,
                    request: 500.0,
                    limit: 1000.0,
                },
                memory: ResourceValues {
                    actual: 128.0,
                    request: 256.0,
                    limit: 512.0,
                },
            },
            Pod {
                name: "worker-queue-a3bcd",
                cpu: ResourceValues {
                    actual: 920.0,
                    request: 1000.0,
                    limit: 1000.0,
                },
                memory: ResourceValues {
                    actual: 480.0,
                    request: 512.0,
                    limit: 512.0,
                },
            },
            Pod {
                name: "ingress-nginx-9k2s0",
                cpu: ResourceValues {
                    actual: 45.0,
                    request: 100.0,
                    limit: 500.0,
                },
                memory: ResourceValues {
                    actual: 64.0,
                    request: 128.0,
                    limit: 256.0,
                },
            },
        ];

        (State { pods, selected: 0 }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Next => {
                state.selected = (state.selected + 1).min(state.pods.len().saturating_sub(1));
            }
            Msg::Prev => {
                state.selected = state.selected.saturating_sub(1);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();

        let rows = Layout::vertical(vec![Constraint::Length(3); state.pods.len()]).split(area);

        for (i, pod) in state.pods.iter().enumerate() {
            let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rows[i]);

            let cpu_state = ResourceGaugeState::default()
                .with_values(pod.cpu)
                .with_label(pod.name)
                .with_title(format!(" {} -- CPU (m) ", pod.name))
                .with_units("m")
                .with_show_legend(true);
            let mem_state = ResourceGaugeState::default()
                .with_values(pod.memory)
                .with_title(format!(" {} -- Memory (Mi) ", pod.name))
                .with_units("Mi")
                .with_show_legend(true);

            let focused = i == state.selected;

            ResourceGauge::view(
                &cpu_state,
                &mut RenderContext::new(frame, cols[0], &theme).focused(focused),
            );
            ResourceGauge::view(
                &mem_state,
                &mut RenderContext::new(frame, cols[1], &theme).focused(focused),
            );
        }
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        match key.code {
            Key::Down | Key::Char('j') => Some(Msg::Next),
            Key::Up | Key::Char('k') => Some(Msg::Prev),
            Key::Char('q') | Key::Esc => Some(Msg::Quit),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<QuotaApp, _>::virtual_builder(90, 12).build()?;

    println!("=== ResourceGauge Example -- K8s pod quota ===\n");

    vt.tick()?;
    println!("Initial: api-server highlighted (350m/500m/1000m CPU, healthy):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Next);
    vt.tick()?;
    println!("worker-queue highlighted (920m/1000m/1000m CPU, near limit -- red):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Next);
    vt.tick()?;
    println!("ingress-nginx highlighted (45m/100m/500m CPU, comfortably under -- green):");
    println!("{}\n", vt.display());

    // Demonstrate the values() accessor round-tripping what with_values() set.
    let sample = ResourceGaugeState::default().with_values(ResourceValues {
        actual: 250.0,
        request: 500.0,
        limit: 1000.0,
    });
    let ResourceValues {
        actual,
        request,
        limit,
    } = sample.values();
    println!("Sample gauge values(): actual={actual}, request={request}, limit={limit}");

    Ok(())
}
