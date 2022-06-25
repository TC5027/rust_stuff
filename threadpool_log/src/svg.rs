// TODO revoir les syntaxes match ? moyen de faire plus succinct ?

use crate::events::*;
use std::io::{Result, Write};
use std::iter::once;
use std::time::Instant;

pub fn display_global_queue<T>(
    mut output: T,
    eventlogs: &[EventLog],
    time_start: Instant,
    svg_width: usize,
    svg_height: usize,
) -> Result<()>
where
    T: Write,
{
    // the global queue is displayed in the top third of the screen
    let global_queue_svg_height = (svg_height / 3) - 20;
    let global_queue_svg_width = svg_width / 2;
    writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:rgb(255,255,255);stroke-width:3;stroke:rgb(0,0,0)\"/>",global_queue_svg_width/2, 10, global_queue_svg_width, global_queue_svg_height)?;
    // we gather in the local deques' eventlogs the AddTasks event
    let filter_addtasks: Vec<Vec<&Event>> = (0..eventlogs.len() - 1)
        .map(|i| {
            eventlogs[i]
                .iter()
                .filter(|event| match event.category {
                    EventCategory::AddTasks(_) => true,
                    _ => false,
                })
                .collect()
        })
        .collect();
    // we sort (according to time) the AddTasks events with the AddRequest events
    // of the global queue's eventlog
    // TODO merge sorted vec, tous déja triés selon le temps
    let mut everything: Vec<&Event> = filter_addtasks
        .into_iter()
        .chain(once(eventlogs[eventlogs.len() - 1].iter().collect()))
        .collect::<Vec<_>>()
        .concat();
    everything.sort_by(|a, b| a.time.cmp(&b.time));
    // we compute the max number of elements there was at same time inside the global queue
    let max_inside_globalqueue = everything
        .iter()
        .map(|event| match event.category {
            EventCategory::AddRequest => 1,
            _ => -1,
        })
        .fold((0, 0), |(current, max), x| {
            let new_current = current + x;
            (new_current, std::cmp::max(max, new_current))
        })
        .1;
    // we deduce the height for our requests
    let request_svg_height = (global_queue_svg_height as isize) / (1 + max_inside_globalqueue);

    // we iterate over the sorted events
    let mut evolving_index = 0;
    for event in everything {
        let time_rounded = event.time.duration_since(time_start).as_millis();
        let color = event.color;
        // if the event is an AddRequest we write a colored rectangle
        // else, if the event is an AddTasks meaning the global queue losts one request,
        // we "erase" it by writing a white rectangle
        if let EventCategory::AddRequest = event.category {
            writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">",global_queue_svg_width/2, 10+evolving_index*request_svg_height, global_queue_svg_width, request_svg_height)?;
            writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"rgb({},{},{})\" begin=\"{}ms\"/>",color.0,color.1,color.2,time_rounded)?;
            evolving_index += 1;
        } else {
            evolving_index -= 1;
            writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">",global_queue_svg_width/2, 10+evolving_index*request_svg_height, global_queue_svg_width, request_svg_height)?;
            writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"white\" begin=\"{}ms\"/>",time_rounded)?;
        }
        writeln!(output, "</rect>")?;
    }
    Ok(())
}

pub fn display_local_deques<T>(
    mut output: T,
    eventlogs: &[EventLog],
    time_start: Instant,
    svg_width: usize,
    svg_height: usize,
) -> Result<()>
where
    T: Write,
{
    // the local deques are displayed in the middle of the screen
    let local_deque_svg_height = (svg_height) / 3 - 20;
    let local_deque_svg_width = ((svg_width - 10) / (eventlogs.len() - 1)) - 10;
    // we compute the max number of tasks there are inside a local deque
    let max_inside_local_deque = (0..eventlogs.len() - 1)
        .into_iter()
        .map(|i| &eventlogs[i])
        .flatten()
        .map(|event| match event.category {
            EventCategory::AddTasks(x) => x,
            _ => 1,
        })
        .max()
        .unwrap();
    // we deduce the height for our tasks
    let task_svg_height = local_deque_svg_height / max_inside_local_deque;
    // we iterate over each local deque's eventlog
    for index in 0..eventlogs.len() - 1 {
        writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:rgb(255,255,255);stroke-width:3;stroke:rgb(0,0,0)\"/>",10+index*(10+local_deque_svg_width),svg_height/3+10,local_deque_svg_width,local_deque_svg_height)?;
        for i in 0..max_inside_local_deque {
            writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\"/>", 10+index*(10+local_deque_svg_width), svg_height/3+10+i*task_svg_height, local_deque_svg_width, task_svg_height)?;
        }
        // we gather the steals affecting this local deque
        let filter_steal: Vec<Vec<&Event>> = (0..eventlogs.len() - 1)
            .into_iter()
            .filter(|i| i != &index)
            .map(|i| {
                eventlogs[i]
                    .iter()
                    .filter(|event| match event.category {
                        EventCategory::Steal(j) => index == j,
                        _ => false,
                    })
                    .collect()
            })
            .collect();
        // we sort (according to time) the Steal events with the events
        // of this local deque's eventlog
        // TODO merge sorted vec, tous déja triés selon le temps
        let mut everything: Vec<&Event> = filter_steal
            .into_iter()
            .chain(once(eventlogs[index].iter().collect()))
            .collect::<Vec<_>>()
            .concat();
        everything.sort_by(|a, b| a.time.cmp(&b.time));

        // we iterate over the sorted events
        let mut evolving_index = 0;
        for event in everything {
            let time_rounded = event.time.duration_since(time_start).as_millis();
            let color = event.color;
            match event.category {
                // we write x colored rectangle
                EventCategory::AddTasks(x) => {
                    for _ in 0..x {
                        writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">", 10+index*(10+local_deque_svg_width), svg_height/3+10+evolving_index*task_svg_height, local_deque_svg_width, task_svg_height)?;
                        writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"rgb({},{},{})\" begin=\"{}ms\"/>",color.0,color.1,color.2,time_rounded)?;
                        writeln!(output, "</rect>")?;
                        evolving_index += 1;
                    }
                }
                // we erase the last rectangle by writing a white rectangle
                EventCategory::StartProcessing => {
                    evolving_index -= 1;
                    writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">", 10+index*(10+local_deque_svg_width), svg_height/3+10+evolving_index*task_svg_height, local_deque_svg_width, task_svg_height)?;
                    writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"white\" begin=\"{}ms\"/>",time_rounded)?;
                    writeln!(output, "</rect>")?;
                }
                // if WE are stolen we erase, otherwise we write a colored rectangle
                EventCategory::Steal(i) => {
                    if i == index {
                        evolving_index -= 1;
                        writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">", 10+index*(10+local_deque_svg_width), svg_height/3+10+evolving_index*task_svg_height, local_deque_svg_width, task_svg_height)?;
                        writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"white\" begin=\"{}ms\"/>",time_rounded)?;
                        writeln!(output, "</rect>")?;
                    } else {
                        writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">", 10+index*(10+local_deque_svg_width), svg_height/3+10+evolving_index*task_svg_height, local_deque_svg_width, task_svg_height)?;
                        writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" values=\"rgb({},{},{})\" begin=\"{}ms\"/>",color.0,color.1,color.2,time_rounded)?;
                        writeln!(output, "</rect>")?;
                        evolving_index += 1;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn display_processing_units<T>(
    mut output: T,
    eventlogs: &[EventLog],
    time_start: Instant,
    svg_width: usize,
    svg_height: usize,
) -> Result<()>
where
    T: Write,
{
    // the processing units are displayed in the bottom third of the screen
    let processing_unit_svg_height = (svg_height) / 3 - 20;
    let processing_unit_svg_width = ((svg_width - 10) / (eventlogs.len() - 1)) - 10;

    // we iterate over each local deque's eventlog
    for index in 0..eventlogs.len() - 1 {
        writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"fill:rgb(255,255,255);stroke-width:3;stroke:rgb(0,0,0)\"/>",10+index*(10+processing_unit_svg_width),2*svg_height/3+10,processing_unit_svg_width,processing_unit_svg_height)?;
        let mut startprocessing_time = 0;
        // we iterate over the events
        for event in &eventlogs[index] {
            let time_rounded = event.time.duration_since(time_start).as_millis();
            let color = event.color;
            if let EventCategory::StartProcessing = event.category {
                startprocessing_time = time_rounded;
            } else if let EventCategory::EndProcessing = event.category {
                // we write a fading rectangle, fading from colored to white
                // during the time of processing
                writeln!(output,"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" style=\"stroke-width:3;stroke:rgb(0,0,0)\">", 10+index*(10+processing_unit_svg_width), 2*svg_height/3+10, processing_unit_svg_width, processing_unit_svg_height)?;
                writeln!(output,"<animate attributeType=\"XML\" attributeName=\"fill\" from=\"rgb({},{},{})\" to=\"white\" dur=\"{}ms\" begin=\"{}ms\"/>",color.0,color.1,color.2,time_rounded-startprocessing_time, startprocessing_time)?;
                writeln!(output, "</rect>")?;
            }
        }
    }

    Ok(())
}
