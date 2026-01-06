use nnnoiseless::DenoiseState;
use crate::config;

pub struct NnnoiselessVAD {
    state: Box<DenoiseState<'static>>,
    buffer: Vec<f32>,
}

impl NnnoiselessVAD {
    pub fn new() -> Self {
        Self {
            state: DenoiseState::new(),
            buffer: Vec::with_capacity(config::NNNOISELESS_FRAME_SIZE * 2),
        }
    }

    pub fn detect(&mut self, input: &[i16]) -> (bool, f32) {
        for &sample in input {
            self.buffer.push(sample as f32);
        }

        let mut total_vad = 0.0f32;
        let mut frame_count = 0u32;

        while self.buffer.len() >= config::NNNOISELESS_FRAME_SIZE {
            let mut input_frame = [0.0f32; 480];
            let mut output_frame = [0.0f32; 480];
            
            input_frame.copy_from_slice(&self.buffer[..config::NNNOISELESS_FRAME_SIZE]);
            self.buffer.drain(..config::NNNOISELESS_FRAME_SIZE);

            let vad_prob = self.state.process_frame(&mut output_frame, &input_frame);
            total_vad += vad_prob;
            frame_count += 1;
        }

        if frame_count == 0 {
            return (true, 0.5);
        }

        let avg_vad = total_vad / frame_count as f32;
        let is_voice = avg_vad >= config::VAD_NNNOISELESS_THRESHOLD;

        (is_voice, avg_vad)
    }

    pub fn reset(&mut self) {
        self.state = DenoiseState::new();
        self.buffer.clear();
    }
}