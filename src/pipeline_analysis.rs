
use std::collections::HashMap;

enum BufferState
{
	Unmapped,
	UsedInSubmission { submission_id : usize }
}

enum SubmissionState
{
	Completed,
	Encoding,
	Submitted,
}

#[derive(Default)]
struct PipelineState
{
	buffer_states : HashMap<usize, BufferState>,
	//old_submission_states : HashMap<usize, SubmissionState>
}

impl PipelineState
{
	/*fn bind_buffer(submission_id : usize, buffer_id : usize)
	{
		if let Some(old_state) = buffer_states.get(& buffer_id)
		{
			match old_state
			{
				BufferState::Unmapped => (),
				BufferState::UsedInSubmission {submission_id : other_submission_id} => 
			}
		}

		if let Some(old_state) = 
		{

		}
	}

	fn submit(submission_id : usize)
	{
		// check that submission is in encoding
		// commit buffers and mark as in submission
		// submit
	}*/
}