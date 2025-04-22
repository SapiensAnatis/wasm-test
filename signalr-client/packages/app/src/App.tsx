import {useState, useEffect, useCallback, FormEvent} from 'react'
import './App.css'


import {ChatClient} from 'signalr-wasm';

import { generate } from 'random-words';

const myUser = generate() as string;
const client = new ChatClient("ws://localhost:5095/chatHub", myUser);

let didInit = false;

function App() {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [messages, setMessages] = useState<{ user: string, message: string }[]>([]);

  const onMessage = useCallback((user: string, message: string) => {
    setMessages((old) => [...old, {user, message}]);
  }, [setMessages]);

  useEffect(() => {
    const func = async () => {
      console.log("Running init");

      if (didInit) {
        return;
      }

      didInit = true;

      try {
        await client.connect();
        client.on_message_received(onMessage)
        console.log("Success");
      } catch (e) {
        console.error(e);
        setError(e as string);
      } finally {
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

    client.send_message(message);
  }, []);


  return (
    <>
      <div style={{width: '100%', flexGrow: 1, display: 'flex', flexDirection: 'column', alignItems: 'start', gap: '10px'}}>
        {messages?.map(({user, message}) => (
          <span style={{alignSelf: user === myUser ? 'end' : undefined}}>
            {user}:{' '}
            {message}
          </span>
        ))}
      </div>
      <form onSubmit={handleSubmit} style={{display: 'flex', flexDirection: 'column', gap: '1rem'}}>
        <label htmlFor="message">Enter a message</label>
        <input name="message" required id="message"/>
        <button type="submit" disabled={loading}>Send</button>
      </form>
    </>)
}

export default App
