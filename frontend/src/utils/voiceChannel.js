/**
 * Minimal WebRTC voice over Genius Clan match WS signaling.
 * Audio only — no video.
 */

const ICE = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ],
};

export class VoiceChannel {
  /**
   * @param {{ matchId: string, isInitiator: boolean, sendSignal: (payload: object) => void }} opts
   */
  constructor({ matchId, isInitiator, sendSignal }) {
    this.matchId = matchId;
    this.isInitiator = isInitiator;
    this.sendSignal = sendSignal;
    this.pc = null;
    this.localStream = null;
    this.remoteAudio = null;
    this.muted = false;
    this.started = false;
  }

  async start() {
    if (this.started) return;
    this.started = true;

    try {
      this.localStream = await navigator.mediaDevices.getUserMedia({
        audio: true,
        video: false,
      });
    } catch (e) {
      this.started = false;
      throw new Error('Microphone permission denied or unavailable');
    }

    this.pc = new RTCPeerConnection(ICE);
    this.localStream.getTracks().forEach((t) => this.pc.addTrack(t, this.localStream));

    this.pc.onicecandidate = (ev) => {
      if (ev.candidate) {
        this.sendSignal({ kind: 'ice', candidate: ev.candidate });
      }
    };

    this.pc.ontrack = (ev) => {
      if (!this.remoteAudio) {
        this.remoteAudio = new Audio();
        this.remoteAudio.autoplay = true;
      }
      this.remoteAudio.srcObject = ev.streams[0];
      this.remoteAudio.play().catch(() => {});
    };

    if (this.isInitiator) {
      const offer = await this.pc.createOffer();
      await this.pc.setLocalDescription(offer);
      this.sendSignal({ kind: 'offer', sdp: this.pc.localDescription });
    }
  }

  async handleSignal(payload) {
    if (!payload || !this.pc) {
      // Late signal before start: if answer/offer, try start first as non-initiator path
      if (payload?.kind === 'offer' && !this.started) {
        await this.startAsAnswerer();
      }
      if (!this.pc) return;
    }

    try {
      if (payload.kind === 'offer') {
        await this.pc.setRemoteDescription(payload.sdp);
        const answer = await this.pc.createAnswer();
        await this.pc.setLocalDescription(answer);
        this.sendSignal({ kind: 'answer', sdp: this.pc.localDescription });
      } else if (payload.kind === 'answer') {
        await this.pc.setRemoteDescription(payload.sdp);
      } else if (payload.kind === 'ice' && payload.candidate) {
        await this.pc.addIceCandidate(payload.candidate);
      }
    } catch (e) {
      console.warn('voice signal error', e);
    }
  }

  async startAsAnswerer() {
    if (this.started) return;
    this.isInitiator = false;
    await this.start();
  }

  setMuted(muted) {
    this.muted = muted;
    if (this.localStream) {
      this.localStream.getAudioTracks().forEach((t) => {
        t.enabled = !muted;
      });
    }
  }

  stop() {
    try {
      this.localStream?.getTracks().forEach((t) => t.stop());
      this.pc?.close();
      if (this.remoteAudio) {
        this.remoteAudio.srcObject = null;
      }
    } catch (_) {}
    this.pc = null;
    this.localStream = null;
    this.started = false;
  }
}
