use kay::{Swarm, ToRandom, Recipient, ActorSystem, Individual, Fate};
use descartes::{Into2d, P3};
use core::geometry::AnyShape;
use core::ui::{UserInterface, VirtualKeyCode};
use super::{CurrentPlan};

#[derive(Copy, Clone, Default)]
pub struct LaneStrokeCanvas;

impl Individual for LaneStrokeCanvas{}

use core::ui::Event3d;
use super::PlanControl::{AddLaneStrokeNode, Materialize, CreateGrid};

impl Recipient<Event3d> for LaneStrokeCanvas {
    fn receive(&mut self, msg: &Event3d) -> Fate {match *msg {
        Event3d::DragStarted{at} => {
            CurrentPlan::id() << AddLaneStrokeNode(at.into_2d(), true);
            Fate::Live
        },
        Event3d::DragFinished{..} => {
            Fate::Live
        },
        Event3d::KeyDown(VirtualKeyCode::Return) => {
            CurrentPlan::id() << Materialize(());
            Fate::Live
        },
        Event3d::KeyDown(VirtualKeyCode::C) => {
            Swarm::<::game::lanes_and_cars::Lane>::all() << ToRandom{
                n_recipients: 5000,
                message: Event3d::DragFinished{from: P3::new(0.0, 0.0, 0.0), to: P3::new(0.0, 0.0, 0.0)
            }};
            Fate::Live
        },
        Event3d::KeyDown(VirtualKeyCode::G) => {
            CurrentPlan::id() << CreateGrid(());
            Fate::Live
        }
        _ => Fate::Live
    }}
}

use core::ui::Add;
use core::ui::Focus;

#[derive(Copy, Clone)]
struct AddToUI;

impl Recipient<AddToUI> for LaneStrokeCanvas {
    fn receive(&mut self, _msg: &AddToUI) -> Fate {
        UserInterface::id() << Add::Interactable3d(LaneStrokeCanvas::id(), AnyShape::Everywhere, 0);
        UserInterface::id() << Focus(LaneStrokeCanvas::id());
        Fate::Live
    }
}

pub fn setup(system: &mut ActorSystem) {
    system.add_individual(LaneStrokeCanvas);
    system.add_inbox::<Event3d, LaneStrokeCanvas>();
    system.add_inbox::<AddToUI, LaneStrokeCanvas>();
    LaneStrokeCanvas::id() << AddToUI;    
}