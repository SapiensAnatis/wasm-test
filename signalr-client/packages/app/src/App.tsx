import { useState, useEffect, useCallback, FormEvent } from 'react'
import './App.css'


import { ChatClient } from 'signalr-wasm';

const client = new ChatClient("ws://localhost:5095/chatHub");

let didInit = false;

function App() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    const func = async () => {
      if (didInit) {
        return;
      }

      didInit = true;

      try {
        await client.connect();
        console.log("Success");
      }
      catch (e) {
        console.error(e);
        setError(e as string);
      }
      finally {
        setLoading(false);
      }
    }

    func();
  }, []);

  const handleSubmit = useCallback((evt: FormEvent<HTMLFormElement>) => {
    evt.preventDefault();

    const form = evt.target as HTMLFormElement;
    const values = new FormData(form);


    const message = values.get("message")?.valueOf();
    if (!message || typeof message !== 'string') {
      return;
    }
    
    client.send_message("wasm", message);

  }, [client]);


  return (
    <>
      <p>
        Loading: {loading ? "Yes" : "No"}
      </p>
      <p>
        Error: {error}
      </p>
      <div>
        <form onSubmit={handleSubmit}>
          <label htmlFor="message">Enter a message</label>
          <input name="message" required id="message" />
          <button type="submit" disabled={loading}>Send</button>
        </form>
      </div>
    </>)
}

export default App
