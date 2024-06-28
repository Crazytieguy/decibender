import "@picocss/pico/css/pico.amber.min.css";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { createEffect, createSignal, onCleanup, onMount } from "solid-js";
import "./App.css";

function App() {
  const [thresholds, setThresholds] = createSignal({
    too_loud: -20.0,
    too_quite: -80.0,
    grace: 8.0,
  });
  const [state, setState] = createSignal("Acceptable");
  const [loudness, setLoudness] = createSignal(-50.0);
  onMount(async () => {
    invoke("init", { initialThresholds: thresholds() });
    const unlistenLoudness = await listen("loudness", (event) => {
      // @ts-ignore  TODO: add zod later
      setLoudness(event.payload.loudness);
    });
    const unlistenState = await listen("state", (event) => {
      // @ts-ignore
      setState(event.payload);
    });
    onCleanup(() => {
      unlistenLoudness();
      unlistenState();
    });
  });
  createEffect(() => {
    emit("thresholds", thresholds());
  });

  return (
    <main class="container">
      <h1>Decibender!</h1>

      <p>Current State: {state()}</p>
      <p>Current Loudness: {loudness().toFixed(2)} dB</p>
      <progress value={loudness() + 100} max={100} />
      <label>
        Too Loud: {thresholds().too_loud} dB
        <input
          type="range"
          name="tooLoud"
          value={thresholds().too_loud}
          onChange={(e) =>
            setThresholds((t) => ({ ...t, too_loud: Number(e.target.value) }))
          }
          min={-100.0}
          max={0.0}
        />
      </label>
      <label>
        Too Quiet: {thresholds().too_quite} dB
        <input
          type="range"
          name="tooQuiet"
          value={thresholds().too_quite}
          onChange={(e) =>
            setThresholds((t) => ({ ...t, too_quite: Number(e.target.value) }))
          }
          min={-100.0}
          max={0.0}
        />
      </label>
      <label>
        Grace: {thresholds().grace} dB
        <input
          type="range"
          name="grace"
          value={thresholds().grace}
          onChange={(e) =>
            setThresholds((t) => ({ ...t, grace: Number(e.target.value) }))
          }
          min={0.0}
          max={20.0}
        />
      </label>
    </main>
  );
}

export default App;
