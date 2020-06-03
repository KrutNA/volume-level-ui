mod handler;
mod types;
pub use handler::PulseHandler;
pub use types::SinkInputData;

#[cfg(feature = "another_updater")]
use types::Counter;

#[cfg(feature = "time")]
use std::time::SystemTime;

use std::rc::Rc;
use std::cell::RefCell;

use pulse::callbacks::ListResult;
use pulse::volume::{Volume, ChannelVolumes};

use crate::button;
use iced::slider;

#[cfg(not(feature = "another_updater"))]
pub fn update_sink_inputs(
    handler: &mut PulseHandler,
    sink_inputs: Rc<RefCell<Vec<SinkInputData>>>,
    sink_input_uis: &mut Vec<(slider::State, button::State)>
) {
    #[cfg(feature = "time")]
    let start = SystemTime::now();
    
    let new_sink_inputs: Rc<RefCell<Vec<SinkInputData>>> = Rc::new(RefCell::new(Vec::new()));
    let new_sink_inputs_ref = new_sink_inputs.clone();
    
    let op = handler.introspect.get_sink_input_info_list(
	move |x| match x {
	    ListResult::Item(item) if item.client.is_some() =>
		new_sink_inputs_ref.borrow_mut().push(item.into()),
	    _ => {}
	});
    handler.wait_for_operation(op);

    let mut sink_inputs = sink_inputs.borrow_mut();
    for i in 0..sink_inputs.len() {
	let new_sink_inputs = new_sink_inputs.borrow();
	
    	let find_result = new_sink_inputs
    	    .iter()
    	    .find(|new| new.id == sink_inputs.get(i).unwrap().id);
	
	if let Some(new) = find_result {
	    sink_inputs.get_mut(i).unwrap().volume = new.volume;
	    sink_inputs.get_mut(i).unwrap().mute   = new.mute;
	} else {
	    sink_inputs.remove(i);
	    sink_input_uis.remove(i);
	}
    }
    
    for new_sink_input in new_sink_inputs.borrow().iter() {
	if sink_inputs.iter().find(|old| old.id == new_sink_input.id).is_none() {
	    sink_inputs.push(new_sink_input.clone());
	    sink_input_uis.push((slider::State::new(), button::State::new()));
	}
    }

    #[cfg(feature = "time")]
    println!("Info update for {} s.", SystemTime::now().duration_since(start).unwrap().as_secs_f64());
}

#[cfg(feature = "another_updater")]
pub fn update_sink_inputs(
    handler: &mut PulseHandler,
    sink_inputs: Rc<RefCell<Vec<SinkInputData>>>,
    sink_input_uis: &mut Vec<(slider::State, button::State)>
) {
    #[cfg(feature = "time")]
    let start = SystemTime::now();

    let sink_inputs_ref = sink_inputs.clone();
    let new_ids         = Rc::new(RefCell::new(Vec::new()));
    let new_ids_ref     = new_ids.clone();
    let appended        = Rc::new(RefCell::new(Counter::default()));
    let appended_ref    = appended.clone();
    
    let op = handler.introspect.get_sink_input_info_list(
	move |x| match x {
	    ListResult::Item(item) if item.client.is_some() => {
		let new_si: SinkInputData = item.into();
		let mut sis = sink_inputs_ref.borrow_mut();
		let result = (0..sis.len())
		    .find(|&i| sis[i].id == new_si.id);
		
		new_ids_ref.borrow_mut().push(new_si.id);
		
		if let Some(index) = result {
		    sis[index].volume = new_si.volume;
		    sis[index].mute   = new_si.mute;
		} else {
		    sis.push(new_si);
		    appended_ref.borrow_mut().inc();
		}
	    }
	    _ => {}
	});
    handler.wait_for_operation(op);

    for _ in 0..appended.borrow().value {
	sink_input_uis.push((slider::State::new(),
			     button::State::new()))
    }

    let mut sink_inputs = sink_inputs.borrow_mut();
    let new_ids = new_ids.borrow();
    for index in 0..sink_inputs.len() {
	if !new_ids.contains(&sink_inputs[index].id) {
	    sink_inputs.remove(index);
	    sink_input_uis.remove(index);
	}
    }

    #[cfg(feature = "time")]
    println!("Info update for {} s.",
	     SystemTime::now().duration_since(start).unwrap().as_secs_f64());

}

pub fn update_sink_input_volume_by_id(
    handler: &mut PulseHandler,
    id: u32,
    volume: u32,
) {
    #[cfg(feature = "time")]
    let start = SystemTime::now();
    
    let mut channel_volumes = ChannelVolumes::default();
    channel_volumes.set(2, Volume(volume));
    
    let op = handler.introspect.set_sink_input_volume(id, &channel_volumes, None);

    handler.wait_for_operation(op);

    #[cfg(feature = "time")]
    println!("Change sink input volume for {} s.",
	     SystemTime::now().duration_since(start).unwrap().as_secs_f64());

}

pub fn update_sink_input_mute_by_id(
    handler: &mut PulseHandler,
    id: u32,
    status: bool,
) {
    #[cfg(feature = "time")]
    let start = SystemTime::now();    
    
    let op = handler.introspect.set_sink_input_mute(id, status, None);
    
    handler.wait_for_operation(op);
    
    #[cfg(feature = "time")]
    println!("Change sink input volume for- {} s.",
	     SystemTime::now().duration_since(start).unwrap().as_secs_f64());
}
