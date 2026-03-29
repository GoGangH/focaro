import { getCurrentWindow } from "@tauri-apps/api/window";
import { Dropdown } from "./pages/Dropdown";
import { Dashboard } from "./pages/Dashboard";
import { Settings } from "./pages/Settings";
import { SaveReferencePage } from "./pages/SaveReferencePage";
import { Onboarding } from "./pages/Onboarding";
import "./App.css";

const windowLabel = getCurrentWindow().label;

if (windowLabel !== "dropdown") {
  document.body.classList.add("dark-bg");
}

function App() {
  if (windowLabel === "dashboard") return <Dashboard />;
  if (windowLabel === "settings") return <Settings />;
  if (windowLabel === "save-reference") return <SaveReferencePage />;
  if (windowLabel === "onboarding") return <Onboarding />;
  return <Dropdown />;
}

export default App;
