import "@picocss/pico/css/pico.min.css";
import { listen } from "@tauri-apps/api/event";
import { createSignal, onCleanup, onMount } from "solid-js";
import "./App.css";

function App() {
  const [thresholds, setThresholds] = createSignal({
    too_loud: -10.0,
    too_quiet: -90.0,
    grace: 6.0,
  });
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
  });
  onCleanup(() => {
    unlisten.forEach((fn) => fn());
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
    </main>
  );
}

export default App;
