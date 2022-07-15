//! Tools for handling state-change events emitted by MPD.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::stream::Stream;
use tokio::sync::mpsc::UnboundedReceiver;

/// Stream of state change events.
///
/// This is emitted by MPD during the client idle loops. You can use this to keep local state such
/// as the current volume or queue in sync with MPD. The stream ending (yielding `None`) indicates
/// that the MPD server closed the connection, after which no more events will be emitted and
/// attempting to send a command will return an error.
///
/// If you don't care about these, you can just drop this receiver.
#[derive(Debug)]
pub struct StateChanges {
    pub(crate) rx: UnboundedReceiver<Result<Subsystem, StateChangeError>>,
}

impl Stream for StateChanges {
    type Item = Result<Subsystem, StateChangeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Just delegate for now
        self.rx.poll_recv(cx)
    }
}

/// Subsystems of MPD which can receive state change notifications.
///
/// Derived from [the documentation](https://www.musicpd.org/doc/html/protocol.html#command-idle),
/// but also includes a catch-all to remain forward-compatible.
#[allow(missing_docs)]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Subsystem {
    Database,
    Message,
    Mixer,
    Options,
    Output,
    Partition,
    Player,
    /// Called `playlist` in the protocol.
    Queue,
    Sticker,
    StoredPlaylist,
    Subscription,
    Update,
    Neighbor,
    Mount,

    /// Catch-all variant used when the above variants do not match. Includes the raw subsystem
    /// from the MPD response.
    Other(Box<str>),
}

impl Subsystem {
    pub(crate) fn from_raw_string(raw: String) -> Self {
        match raw.as_str() {
            "database" => Subsystem::Database,
            "message" => Subsystem::Message,
            "mixer" => Subsystem::Mixer,
            "options" => Subsystem::Options,
            "output" => Subsystem::Output,
            "partition" => Subsystem::Partition,
            "player" => Subsystem::Player,
            "playlist" => Subsystem::Queue,
            "sticker" => Subsystem::Sticker,
            "stored_playlist" => Subsystem::StoredPlaylist,
            "subscription" => Subsystem::Subscription,
            "update" => Subsystem::Update,
            "neighbor" => Subsystem::Neighbor,
            "mount" => Subsystem::Mount,
            _ => Subsystem::Other(raw.into()),
        }
    }

    /// Returns the raw protocol name used for this subsystem.
    pub fn as_str(&self) -> &str {
        match self {
            Subsystem::Database => "database",
            Subsystem::Message => "message",
            Subsystem::Mixer => "mixer",
            Subsystem::Options => "options",
            Subsystem::Output => "output",
            Subsystem::Partition => "partition",
            Subsystem::Player => "player",
            Subsystem::Queue => "playlist",
            Subsystem::Sticker => "sticker",
            Subsystem::StoredPlaylist => "stored_playlist",
            Subsystem::Subscription => "subscription",
            Subsystem::Update => "update",
            Subsystem::Neighbor => "neighbor",
            Subsystem::Mount => "mount",
            Subsystem::Other(r) => r,
        }
    }
}
use std::{error, fmt};

use mpd_protocol::{response::Error, MpdProtocolError};

/// Errors which may occur while listening for state change events.
#[derive(Debug)]
pub enum StateChangeError {
    /// An underlying protocol error occurred, including IO errors
    Protocol(MpdProtocolError),
    /// The state change message contained an error frame
    ErrorMessage(Error),
}

impl fmt::Display for StateChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateChangeError::Protocol(_) => write!(f, "protocol error"),
            StateChangeError::ErrorMessage(Error { code, message, .. }) => write!(
                f,
                "message contained an error frame [code {}]: {}",
                code, message
            ),
        }
    }
}

impl error::Error for StateChangeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            StateChangeError::Protocol(e) => Some(e),
            _ => None,
        }
    }
}

#[doc(hidden)]
impl From<Error> for StateChangeError {
    fn from(r: Error) -> Self {
        StateChangeError::ErrorMessage(r)
    }
}

#[doc(hidden)]
impl From<MpdProtocolError> for StateChangeError {
    fn from(e: MpdProtocolError) -> Self {
        StateChangeError::Protocol(e)
    }
}
