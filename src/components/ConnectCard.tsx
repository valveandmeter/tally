import { useState } from "react";
import { connectWrike } from "../lib/api";

interface Props {
  onConnected: (name: string) => void;
}

export function ConnectCard({ onConnected }: Props) {
  const [token, setToken] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);

  const handleConnect = async () => {
    setError(null);
    setConnecting(true);
    try {
      const account = await connectWrike(token);
      onConnected(account.name);
    } catch (e) {
      setError(typeof e === "string" ? e : "Could not connect to Wrike.");
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="connect-card">
      <p className="connect-card__title">Connect to Wrike</p>
      <p className="connect-card__hint">
        Paste a permanent access token from Wrike → Apps &amp; Integrations → API.
      </p>
      <input
        className="connect-card__input"
        type="password"
        value={token}
        onChange={(e) => setToken(e.currentTarget.value)}
        placeholder="Wrike permanent access token"
        onKeyDown={(e) => {
          if (e.key === "Enter" && !connecting) handleConnect();
        }}
      />
      {error && <p className="connect-card__error">{error}</p>}
      <button className="connect-card__button" onClick={handleConnect} disabled={connecting || !token.trim()}>
        {connecting ? "Connecting…" : "Connect"}
      </button>
    </div>
  );
}
