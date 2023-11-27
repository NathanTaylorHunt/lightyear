use bevy::prelude::{Events, Mut, Res, ResMut, Time, World};
use tracing::{debug, trace};

use crate::connection::events::IterEntitySpawnEvent;
use crate::replication::ReplicationSend;
use crate::server::events::{ConnectEvent, DisconnectEvent};
use crate::server::Server;
use crate::shared::events::EntitySpawnEvent;
use crate::{ClientId, MessageProtocol, Protocol};

pub(crate) fn receive<P: Protocol>(world: &mut World) {
    trace!("Receive client packets");
    world.resource_scope(|world, mut server: Mut<Server<P>>| {
        let time = world.get_resource::<Time>().unwrap();

        // update client state, send keep-alives, receive packets from io
        server.update(time.delta()).unwrap();
        // buffer packets into message managers
        server.recv_packets().unwrap();

        // receive events
        server.receive(world);
        // let mut events = server.receive(world);

        // Write the received events into bevy events
        if !server.events.is_empty() {
            // TODO: write these as systems? might be easier to also add the events to the app
            //  it might just be less efficient? + maybe tricky to
            // Input events
            // Update the input buffers with any InputMessage received:

            // ADD A FUNCTION THAT ITERATES THROUGH EACH CONNECTION AND RETURNS InputEvent for THE CURRENT TICK

            // Connection / Disconnection events
            if server.events.has_connections() {
                let mut connect_event_writer =
                    world.get_resource_mut::<Events<ConnectEvent>>().unwrap();
                for client_id in server.events.iter_connections() {
                    debug!("Client connected event: {}", client_id);
                    connect_event_writer.send(ConnectEvent::new(client_id));
                }
            }

            if server.events.has_disconnections() {
                let mut connect_event_writer =
                    world.get_resource_mut::<Events<DisconnectEvent>>().unwrap();
                for client_id in server.events.iter_disconnections() {
                    debug!("Client connected event: {}", client_id);
                    connect_event_writer.send(DisconnectEvent::new(client_id));
                }
            }

            // Message Events
            P::Message::push_message_events(world, &mut server.events);

            // Entity Spawn Events
            if server.events.has_entity_spawn() {
                let mut entity_spawn_event_writer = world
                    .get_resource_mut::<Events<EntitySpawnEvent<ClientId>>>()
                    .unwrap();
                for (entity, client_id) in server.events.into_iter_entity_spawn() {
                    entity_spawn_event_writer.send(EntitySpawnEvent::new(entity, client_id));
                }
            }
        }
    });
}

// or do additional send stuff here
pub(crate) fn send<P: Protocol>(mut server: ResMut<Server<P>>) {
    trace!("Send packets to clients");
    // finalize any packets that are needed for replication
    server.prepare_replicate_send();
    // send buffered packets to io
    server.send_packets().unwrap();
}

pub(crate) fn clear_events<P: Protocol>(mut server: ResMut<Server<P>>) {
    server.clear_events();
}

pub(crate) fn is_ready_to_send<P: Protocol>(server: Res<Server<P>>) -> bool {
    server.is_ready_to_send()
}