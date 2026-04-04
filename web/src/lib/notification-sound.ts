/**
 * Plays a pleasant notification sound when the assistant finishes responding.
 * Uses Web Audio API to generate a more polished, musical chime.
 */
export const playNotificationSound = (): void => {
  try {
    // Type definition for webkit prefixed AudioContext (Safari support)
    interface WindowWithWebkit extends Window {
      webkitAudioContext?: typeof AudioContext;
    }

    const AudioContextClass = window.AudioContext || (window as WindowWithWebkit).webkitAudioContext;
    if (!AudioContextClass) {
      console.warn('AudioContext not supported in this browser');
      return;
    }

    const audioContext = new AudioContextClass();

    // Create three oscillators for a richer chime
    const oscillator1 = audioContext.createOscillator();
    const oscillator2 = audioContext.createOscillator();
    const oscillator3 = audioContext.createOscillator();

    const gainNode1 = audioContext.createGain();
    const gainNode2 = audioContext.createGain();
    const gainNode3 = audioContext.createGain();

    // Set frequencies for a major chord (C5, E5, G5)
    oscillator1.type = 'sine';
    oscillator1.frequency.setValueAtTime(523.25, audioContext.currentTime); // C5

    oscillator2.type = 'sine';
    oscillator2.frequency.setValueAtTime(659.25, audioContext.currentTime); // E5

    oscillator3.type = 'sine';
    oscillator3.frequency.setValueAtTime(783.99, audioContext.currentTime); // G5

    // Set up envelope for smooth sound
    const now = audioContext.currentTime;
    gainNode1.gain.setValueAtTime(0, now);
    gainNode1.gain.linearRampToValueAtTime(0.15, now + 0.02);
    gainNode1.gain.exponentialRampToValueAtTime(0.01, now + 0.5);

    gainNode2.gain.setValueAtTime(0, now);
    gainNode2.gain.linearRampToValueAtTime(0.12, now + 0.02);
    gainNode2.gain.exponentialRampToValueAtTime(0.01, now + 0.5);

    gainNode3.gain.setValueAtTime(0, now);
    gainNode3.gain.linearRampToValueAtTime(0.1, now + 0.02);
    gainNode3.gain.exponentialRampToValueAtTime(0.01, now + 0.5);

    // Connect audio nodes
    oscillator1.connect(gainNode1);
    oscillator2.connect(gainNode2);
    oscillator3.connect(gainNode3);
    gainNode1.connect(audioContext.destination);
    gainNode2.connect(audioContext.destination);
    gainNode3.connect(audioContext.destination);

    // Play the sound
    oscillator1.start(now);
    oscillator2.start(now + 0.05);
    oscillator3.start(now + 0.1);

    // Stop after sound completes
    oscillator1.stop(now + 0.5);
    oscillator2.stop(now + 0.55);
    oscillator3.stop(now + 0.6);

    // Clean up
    setTimeout(() => {
      audioContext.close();
    }, 1000);
  } catch (error) {
    console.error('Failed to play notification sound:', error);
  }
};