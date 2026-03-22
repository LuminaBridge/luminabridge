// WebSocket Test Script for LuminaBridge
import { WebSocket } from 'ws';

const ws = new WebSocket('ws://localhost:3000/api/v1/ws');

ws.on('open', () => {
  console.log('✅ WebSocket connected');
});

ws.on('message', (data) => {
  const message = JSON.parse(data);
  console.log('📊 Received stats:', message.data);
  
  // Close after receiving 3 messages
  if (!this.count) this.count = 0;
  this.count++;
  
  if (this.count >= 3) {
    console.log('✅ Received 3 messages, closing connection');
    ws.close();
    process.exit(0);
  }
});

ws.on('error', (error) => {
  console.error('❌ WebSocket error:', error.message);
  process.exit(1);
});

ws.on('close', () => {
  console.log('👋 WebSocket closed');
});

// Timeout after 10 seconds
setTimeout(() => {
  console.log('⏱️ Timeout reached');
  process.exit(0);
}, 10000);
