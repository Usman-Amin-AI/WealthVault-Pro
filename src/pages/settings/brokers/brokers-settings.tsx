import { useState, useEffect } from "react";
import {
  Button,
  Input,
  Separator,
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@investwise/ui";
import { toast } from "sonner";
import { getSettings, updateSettings } from "@/commands/settings";
import { SettingsHeader } from "../settings-header";

export default function BrokersSettingsPage() {
  const [ibkrClientId, setIbkrClientId] = useState("");
  const [ibkrClientSecret, setIbkrClientSecret] = useState("");
  const [schwabClientId, setSchwabClientId] = useState("");
  const [schwabClientSecret, setSchwabClientSecret] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const settings = await getSettings();
      const settingsRecord = settings as unknown as Record<string, string | undefined>;
      setIbkrClientId(settingsRecord["IBKR_CLIENT_ID"] ?? "");
      setIbkrClientSecret(settingsRecord["IBKR_CLIENT_SECRET"] ?? "");
      setSchwabClientId(settingsRecord["SCHWAB_CLIENT_ID"] ?? "");
      setSchwabClientSecret(settingsRecord["SCHWAB_CLIENT_SECRET"] ?? "");
    } catch (err) {
      console.error(err);
      toast.error("Failed to load broker settings");
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (provider: string, id: string, secret: string) => {
    try {
      const payload = {
        [`${provider}_CLIENT_ID`]: id,
        [`${provider}_CLIENT_SECRET`]: secret,
      };
      await updateSettings(payload);
      toast.success(`${provider} API keys saved successfully.`);
    } catch (err) {
      console.error(err);
      toast.error(`Failed to save ${provider} API keys.`);
    }
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div className="space-y-6">
      <SettingsHeader
        heading="Broker Integrations"
        text="Configure developer API keys to connect to your brokers."
      />
      <Separator />

      <div className="grid gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Interactive Brokers (IBKR)</CardTitle>
            <CardDescription>
              Enter your IBKR API Client ID and Secret to enable OAuth connections.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Client ID</label>
              <Input
                value={ibkrClientId}
                onChange={(e) => setIbkrClientId(e.target.value)}
                placeholder="IBKR Client ID"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Client Secret</label>
              <Input
                type="password"
                value={ibkrClientSecret}
                onChange={(e) => setIbkrClientSecret(e.target.value)}
                placeholder="IBKR Client Secret"
              />
            </div>
            <Button onClick={() => handleSave("IBKR", ibkrClientId, ibkrClientSecret)}>
              Save IBKR Keys
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Charles Schwab</CardTitle>
            <CardDescription>Enter your Schwab Developer API Client ID and Secret.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Client ID</label>
              <Input
                value={schwabClientId}
                onChange={(e) => setSchwabClientId(e.target.value)}
                placeholder="Schwab Client ID"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Client Secret</label>
              <Input
                type="password"
                value={schwabClientSecret}
                onChange={(e) => setSchwabClientSecret(e.target.value)}
                placeholder="Schwab Client Secret"
              />
            </div>
            <Button onClick={() => handleSave("SCHWAB", schwabClientId, schwabClientSecret)}>
              Save Schwab Keys
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
