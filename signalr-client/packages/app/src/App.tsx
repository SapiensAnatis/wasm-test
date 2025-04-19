import { useState, useEffect } from 'react'
import './App.css'


import * as ChatClient from 'chat-client';

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
        await ChatClient.promise();
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
