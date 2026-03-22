import { useState } from 'react'

function App() {
  const [health, setHealth] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)

  const checkHealth = async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await fetch('/health')
      const data = await response.json()
      setHealth(data)
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div style={{ 
      minHeight: '100vh', 
      display: 'flex', 
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      color: 'white',
      padding: '20px'
    }}>
      <div style={{ 
        textAlign: 'center',
        background: 'rgba(255, 255, 255, 0.1)',
        backdropFilter: 'blur(10px)',
        borderRadius: '20px',
        padding: '40px',
        maxWidth: '500px',
        width: '100%'
      }}>
        <h1 style={{ fontSize: '2.5rem', marginBottom: '10px' }}>
          🌉 LuminaBridge
        </h1>
        <p style={{ fontSize: '1.2rem', opacity: 0.9, marginBottom: '30px' }}>
          Illuminating AI Connections
        </p>
        
        <button
          onClick={checkHealth}
          disabled={loading}
          style={{
            padding: '12px 30px',
            fontSize: '1rem',
            background: 'white',
            color: '#667eea',
            border: 'none',
            borderRadius: '8px',
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading ? 0.7 : 1,
            transition: 'all 0.3s ease',
            fontWeight: 'bold'
          }}
        >
          {loading ? 'Checking...' : 'Check Backend Health'}
        </button>

        {health && (
          <div style={{ 
            marginTop: '30px',
            padding: '20px',
            background: 'rgba(72, 187, 120, 0.3)',
            borderRadius: '10px',
            border: '1px solid rgba(72, 187, 120, 0.5)'
          }}>
            <h3 style={{ margin: '0 0 10px 0' }}>✅ Backend Status</h3>
            <pre style={{ 
              margin: 0, 
              textAlign: 'left',
              fontSize: '0.9rem',
              background: 'rgba(0, 0, 0, 0.2)',
              padding: '10px',
              borderRadius: '5px',
              overflow: 'auto'
            }}>
              {JSON.stringify(health, null, 2)}
            </pre>
          </div>
        )}

        {error && (
          <div style={{ 
            marginTop: '30px',
            padding: '20px',
            background: 'rgba(239, 68, 68, 0.3)',
            borderRadius: '10px',
            border: '1px solid rgba(239, 68, 68, 0.5)'
          }}>
            <h3 style={{ margin: '0 0 10px 0' }}>❌ Connection Error</h3>
            <p style={{ margin: 0 }}>{error}</p>
          </div>
        )}

        <div style={{ 
          marginTop: '40px',
          paddingTop: '20px',
          borderTop: '1px solid rgba(255, 255, 255, 0.2)',
          fontSize: '0.85rem',
          opacity: 0.7
        }}>
          <p>Frontend is running successfully!</p>
          <p>Backend API: <code style={{ background: 'rgba(0,0,0,0.3)', padding: '2px 8px', borderRadius: '4px' }}>http://localhost:3000</code></p>
        </div>
      </div>
    </div>
  )
}

export default App
