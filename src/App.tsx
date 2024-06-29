import "@picocss/pico/css/pico.min.css";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { createEffect, createSignal, onCleanup, onMount } from "solid-js";
import "./App.css";

function App() {
  const [thresholds, setThresholds] = createSignal({
    too_loud: -10.0,
    too_quiet: -90.0,
    grace: 6.0,
  });
  const [rmsSeconds, setRmsSeconds] = createSignal(3);
  const [state, setState] = createSignal("Acceptable");
  const [loudness, setLoudness] = createSignal(-50.0);
  const unlisten: (() => void)[] = [];
  onMount(async () => {
    unlisten.push(
      ...(await Promise.all([
        await listen("loudness", (event) => {
          // @ts-ignore  TODO: add zod later
          setLoudness(event.payload.loudness);
        }),
        await listen("state", (event) => {
          // @ts-ignore
          setState(event.payload);
        }),
        await listen("thresholds", (event) => {
          // @ts-ignore
          setThresholds(event.payload);
        }),
      ]))
    );
    invoke("init", { initialRmsSeconds: rmsSeconds() });
  });
  onCleanup(() => {
    unlisten.forEach((fn) => fn());
  });
  createEffect(() => {
    emit("rms-seconds", { rms_seconds: rmsSeconds() });
  });

  return (
    <main class="container">
      <h1>Decibender!</h1>

      <p>Current State: {state()}</p>
      <div class="progress-container">
        <progress
          value={loudness() + 100}
          max={100}
          class="absolute progress-height"
        />
        <progress
          value={thresholds().too_quiet + 100}
          max={100}
          class="absolute threshold progress-height"
        />
        <progress
          value={thresholds().too_quiet + thresholds().grace + 100}
          max={100}
          class="absolute progress-height grace"
        />
        <progress
          value={-thresholds().too_loud}
          max={100}
          class="absolute threshold progress-height rotate-180"
        />
        <progress
          value={-thresholds().too_loud + thresholds().grace}
          max={100}
          class="absolute progress-height grace rotate-180"
        />
        <span class="absolute left">
          {(thresholds().too_quiet + 100).toFixed(1)} dB
        </span>
        <span class="absolute center">{(loudness() + 100).toFixed(1)} dB</span>
        <span class="absolute right">
          {(thresholds().too_loud + 100).toFixed(1)} dB
        </span>
      </div>
      <label>
        RMS Seconds: {rmsSeconds()}
        <input
          type="range"
          name="rmsSeconds"
          value={rmsSeconds()}
          onChange={(e) => setRmsSeconds(Number(e.target.value))}
          step={0.5}
          min={1}
          max={10}
        />
      </label>
      <div class="grid">
        <button onClick={() => emit("louder")}>Louder!</button>
        <button onClick={() => emit("quieter")}>Quieter!</button>
      </div>
    </main>
  );
}

export default App;
