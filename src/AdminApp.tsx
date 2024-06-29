import "@picocss/pico/css/pico.min.css";
import { emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { createEffect, createSignal, onMount } from "solid-js";

function App() {
  const [thresholds, setThresholds] = createSignal({
    too_loud: -35.0,
    too_quiet: -75.0,
    grace: 6.0,
  });
  const [rmsSeconds, setRmsSeconds] = createSignal(3);

  onMount(async () => {
    invoke("init", {
      initialRmsSeconds: rmsSeconds(),
      initialThresholds: thresholds(),
    });
  });
  createEffect(() => {
    emit("rms-seconds", { rms_seconds: rmsSeconds() });
  });
  createEffect(() => {
    emit("thresholds", thresholds());
  });

  return (
    <main class="container">
      <h1 style="margin-top: 1rem;">Decibender Admin!</h1>
      <div class="grid">
        <label>
          Too Quiet:
          <input
            type="number"
            name="tooQuiet"
            value={thresholds().too_quiet}
            onChange={(e) =>
              setThresholds((current) => ({
                too_loud: current.too_loud,
                too_quiet: Number(e.target.value),
                grace: current.grace,
              }))
            }
            step={0.5}
            min={-100}
            max={0}
          />
        </label>
        <label>
          Too Loud:
          <input
            type="number"
            name="tooLoud"
            value={thresholds().too_loud}
            onChange={(e) =>
              setThresholds((current) => ({
                too_loud: Number(e.target.value),
                too_quiet: current.too_quiet,
                grace: current.grace,
              }))
            }
            step={0.5}
            min={0}
            max={100}
          />
        </label>
      </div>
      <div class="grid">
        <label>
          Grace:
          <input
            type="number"
            name="grace"
            value={thresholds().grace}
            onChange={(e) =>
              setThresholds((current) => ({
                too_loud: current.too_loud,
                too_quiet: current.too_quiet,
                grace: Number(e.target.value),
              }))
            }
            step={0.5}
            min={0}
            max={20}
          />
        </label>
        <label>
          RMS Seconds:
          <input
            type="number"
            name="rmsSeconds"
            value={rmsSeconds()}
            onChange={(e) => setRmsSeconds(Number(e.target.value))}
            step={0.5}
            min={0.5}
            max={10}
          />
        </label>
      </div>
      <div class="grid">
        <button
          onClick={async () => {
            await emit("louder");
            setThresholds((current) => ({
              too_loud: current.too_loud + 6.0,
              too_quiet: current.too_quiet + 6.0,
              grace: current.grace,
            }));
          }}
        >
          Louder!
        </button>
        <button
          onClick={async () => {
            await emit("quieter");
            setThresholds((current) => ({
              too_loud: current.too_loud - 6.0,
              too_quiet: current.too_quiet - 6.0,
              grace: current.grace,
            }));
          }}
        >
          Quieter!
        </button>
      </div>
    </main>
  );
}

export default App;
