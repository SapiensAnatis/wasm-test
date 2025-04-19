import { useState, useEffect } from 'react'
import './App.css'


import {ChatClient } from 'chat-client';

const client = new ChatClient("ws://localhost:5095/chatHub");

let didInit = false;

function App() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");


  useEffect(() => {
    const func = async () => {
      if (didInit) {
        return;
      }

      didInit = true;
      setLoading(true);
      
      try {
        await client.connect();
        console.log("Success");
        await client.infinite_read();
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


  return (
    <>
      <p>
        Loading: {loading ? "Yes" : "No"}
      </p>
      <p>
        Error: {error}
      </p>
    </>)
}

export default App
