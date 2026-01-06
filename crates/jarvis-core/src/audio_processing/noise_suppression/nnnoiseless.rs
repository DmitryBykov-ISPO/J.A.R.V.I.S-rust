use nnnoiseless::DenoiseState;
use crate::config;

pub struct NnnoiselessNS {
    state: Box<DenoiseState<'static>>,
    buffer: Vec<f32>,
}

impl NnnoiselessNS {
    pub fn new() -> Self {
        Self {
            state: DenoiseState::new(),
            buffer: Vec::with_capacity(config::NNNOISELESS_FRAME_SIZE * 2),
        }
    }

    pub fn process(&mut self, input: &[i16]) -> Vec<i16> {
        for &sample in input {
            self.buffer.push(sample as f32);
        }

        let mut output: Vec<i16> = Vec::with_capacity(input.len());

        while self.buffer.len() >= config::NNNOISELESS_FRAME_SIZE {
            let mut input_frame = [0.0f32; 480];
            let mut output_frame = [0.0f32; 480];
            
            input_frame.copy_from_slice(&self.buffer[..config::NNNOISELESS_FRAME_SIZE]);
            self.buffer.drain(..config::NNNOISELESS_FRAME_SIZE);

            // process: input -> output (denoised)
            let _ = self.state.process_frame(&mut output_frame, &input_frame);

            for &sample in &output_frame {
                let clamped = sample.clamp(i16::MIN as f32, i16::MAX as f32);
                output.push(clamped as i16);
            }
        }

        if output.is_empty() {
            return input.to_vec();
        }

        output
    }

    pub fn reset(&mut self) {
        self.state = DenoiseState::new();
        self.buffer.clear();
    }
}