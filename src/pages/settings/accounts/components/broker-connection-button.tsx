import { useState, useEffect } from "react";
import { Button, Icons } from "@investwise/ui";
import { toast } from "sonner";
import {
  getAuthUrl,
  handleOauthCallback,
  getBrokerConnection,
  fetchBrokerActivities,
  BrokerConnectionDto,
} from "@/commands/brokers";
import { importActivities } from "@/commands/activity-import";

interface BrokerConnectionButtonProps {
  accountId: string;
  profileId?: string;
}

export function BrokerConnectionButton({ accountId, profileId }: BrokerConnectionButtonProps) {
  const [connection, setConnection] = useState<BrokerConnectionDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [connecting, setConnecting] = useState(false);

  useEffect(() => {
    loadConnection();
  }, [accountId, profileId]);

  const loadConnection = async () => {
    try {
      const conn = await getBrokerConnection(accountId, profileId);
      setConnection(conn);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = async (provider: string) => {
    try {
      setConnecting(true);
      const url = await getAuthUrl(provider, profileId);

      // Attempt to open the URL directly if we are in a web context or Tauri handles it
      window.open(url, "_blank");

      toast.info("Waiting for authentication...", {
        description: "Please complete the login in your browser.",
      });

      const newConn = await handleOauthCallback(accountId, provider, profileId);
      setConnection(newConn);
      toast.success("Broker Connected", {
        description: `Successfully connected to ${provider}`,
      });
    } catch (error: any) {
      toast.error("Connection Failed", {
        description: error.message || String(error),
      });
    } finally {
      setConnecting(false);
    }
  };

  const handleSync = async () => {
    if (!connection) return;
    try {
      setSyncing(true);
      const activities = await fetchBrokerActivities(accountId, profileId);

      if (activities.length > 0) {
        await importActivities({ activities });
      }

      toast.success("Sync Complete", {
        description: `Imported ${activities.length} new activities.`,
      });
    } catch (error: any) {
      toast.error("Sync Failed", {
        description: error.message || String(error),
      });
    } finally {
      setSyncing(false);
    }
  };

  if (loading) {
    return (
      <Button variant="outline" size="sm" disabled>
        Loading...
      </Button>
    );
  }

  if (connection) {
    return (
      <Button variant="outline" size="sm" onClick={handleSync} disabled={syncing}>
        <Icons.Refresh className={`mr-2 h-4 w-4 ${syncing ? "animate-spin" : ""}`} />
        {syncing ? "Syncing..." : `Sync ${connection.provider}`}
      </Button>
    );
  }

  return (
    <div className="flex gap-2">
      <Button
        variant="outline"
        size="sm"
        onClick={() => handleConnect("IBKR")}
        disabled={connecting}
      >
        <Icons.Link className="mr-2 h-4 w-4" />
        IBKR
      </Button>
      <Button
        variant="outline"
        size="sm"
        onClick={() => handleConnect("SCHWAB")}
        disabled={connecting}
      >
        <Icons.Link className="mr-2 h-4 w-4" />
        Schwab
      </Button>
    </div>
  );
}
