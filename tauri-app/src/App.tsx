import { useState, useEffect } from "react";
import { Tabs, Button } from "./components/ui/primitives";
import ProfileList from "./components/ProfileList";
import RuleEditor from "./components/RuleEditor";
import StatusBar from "./components/StatusBar";
import DoctorView from "./components/DoctorView";
import HelpPanel from "./components/HelpPanel";
import SetupWizard from "./components/SetupWizard";
import { getProfiles } from "./lib/tauri-api";
import {
  LayoutDashboard,
  Users,
  GitFork,
  Stethoscope,
  HelpCircle,
  Moon,
  Sun,
} from "lucide-react";

const tabs = [
  {
    id: "dashboard",
    label: "Dashboard",
    icon: <LayoutDashboard className="h-4 w-4" />,
  },
  {
    id: "profiles",
    label: "Profiles",
    icon: <Users className="h-4 w-4" />,
  },
  {
    id: "rules",
    label: "Rules",
    icon: <GitFork className="h-4 w-4" />,
  },
  {
    id: "doctor",
    label: "Doctor",
    icon: <Stethoscope className="h-4 w-4" />,
  },
  {
    id: "help",
    label: "Help",
    icon: <HelpCircle className="h-4 w-4" />,
  },
];

export default function App() {
  const [activeTab, setActiveTab] = useState("dashboard");
  const [showWizard, setShowWizard] = useState<boolean | null>(null);
  const [darkMode, setDarkMode] = useState(() => {
    if (typeof window !== "undefined") {
      return document.documentElement.classList.contains("dark") ||
        window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    return false;
  });

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [darkMode]);

  // Check if profiles exist — if not, show the setup wizard
  useEffect(() => {
    getProfiles()
      .then((profiles) => {
        setShowWizard(Object.keys(profiles).length === 0);
      })
      .catch(() => {
        setShowWizard(true);
      });
  }, []);

  // Loading state while checking
  if (showWizard === null) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center animate-pulse">
          <span className="text-primary-foreground font-bold text-sm">G</span>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">G</span>
            </div>
            <div>
              <h1 className="text-lg font-semibold leading-none">GitID</h1>
              <p className="text-xs text-muted-foreground">
                Multi-profile Git identity manager
              </p>
            </div>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setDarkMode(!darkMode)}
            aria-label={darkMode ? "Switch to light mode" : "Switch to dark mode"}
          >
            {darkMode ? (
              <Sun className="h-4 w-4" />
            ) : (
              <Moon className="h-4 w-4" />
            )}
          </Button>
        </div>
      </header>

      {showWizard ? (
        /* First-run setup wizard */
        <main className="px-6 py-6">
          <SetupWizard onComplete={() => setShowWizard(false)} />
        </main>
      ) : (
        <>
          {/* Navigation */}
          <div className="px-6">
            <Tabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} />
          </div>

          {/* Content */}
          <main className="px-6 py-6 max-w-4xl" role="tabpanel" id={`tabpanel-${activeTab}`}>
            {activeTab === "dashboard" && <StatusBar />}
            {activeTab === "profiles" && <ProfileList />}
            {activeTab === "rules" && <RuleEditor />}
            {activeTab === "doctor" && <DoctorView />}
            {activeTab === "help" && <HelpPanel />}
          </main>
        </>
      )}
    </div>
  );
}
