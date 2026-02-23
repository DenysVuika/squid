/**
 * Plays a pleasant notification sound when the assistant finishes responding.
 * Uses Web Audio API to generate a sound programmatically.
 */
export const playNotificationSound = (): void => {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    
    // Create two oscillators for a pleasant two-tone chime
    const oscillator1 = audioContext.createOscillator();
    const oscillator2 = audioContext.createOscillator();
    
    const gainNode1 = audioContext.createGain();
    const gainNode2 = audioContext.createGain();
    
    // First tone: 800Hz
    oscillator1.type = 'sine';
    oscillator1.frequency.setValueAtTime(800, audioContext.currentTime);
    
    // Second tone: 1000Hz (perfect fifth interval)
    oscillator2.type = 'sine';
    oscillator2.frequency.setValueAtTime(1000, audioContext.currentTime);
    
    // Set up envelope for smooth sound
    const now = audioContext.currentTime;
    gainNode1.gain.setValueAtTime(0, now);
    gainNode1.gain.linearRampToValueAtTime(0.15, now + 0.02);
    gainNode1.gain.exponentialRampToValueAtTime(0.01, now + 0.3);
    
    gainNode2.gain.setValueAtTime(0, now);
    gainNode2.gain.linearRampToValueAtTime(0.12, now + 0.02);
    gainNode2.gain.exponentialRampToValueAtTime(0.01, now + 0.4);
    
    // Connect audio nodes
    oscillator1.connect(gainNode1);
    oscillator2.connect(gainNode2);
    gainNode1.connect(audioContext.destination);
    gainNode2.connect(audioContext.destination);
    
    // Play the sound
    oscillator1.start(now);
    oscillator2.start(now + 0.05); // Slight delay for second tone
    
    // Stop after sound completes
    oscillator1.stop(now + 0.3);
    oscillator2.stop(now + 0.45);
    
    // Clean up
    setTimeout(() => {
      audioContext.close();
    }, 500);
  } catch (error) {
    console.error('Failed to play notification sound:', error);
  }
};
