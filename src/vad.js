class VoiceActivityDetector {
    constructor(options = {}) {
        this.voiceDetected = false;
        this.threshold = options.threshold || 0.2;
        this.voiceCounter = 0;
        this.silenceCounter = 0;
        this.minVoiceFrames = options.minVoiceFrames || 3;  // Minimum consecutive voice frames
        this.minSilenceFrames = options.minSilenceFrames || 5;  // Minimum consecutive silence frames
    }

    // Process audio data and return true if voice is detected
    process(audioData) {
        // Calculate RMS (Root Mean Square) of the audio buffer
        const rms = Math.sqrt(audioData.reduce((acc, val) => acc + (val * val), 0) / audioData.length);
        
        // Check if current frame contains voice
        const isVoiceFrame = rms > this.threshold;
        
        if (isVoiceFrame) {
            this.voiceCounter++;
            this.silenceCounter = 0;
            if (this.voiceCounter >= this.minVoiceFrames) {
                this.voiceDetected = true;
            }
        } else {
            this.silenceCounter++;
            this.voiceCounter = 0;
            if (this.silenceCounter >= this.minSilenceFrames) {
                this.voiceDetected = false;
            }
        }
        
        return this.voiceDetected;
    }

    // Reset the detector state
    reset() {
        this.voiceDetected = false;
        this.voiceCounter = 0;
        this.silenceCounter = 0;
    }
}
