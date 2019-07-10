pub struct AfterTouchMarker;

pub struct AfterTouchContext<E> {
    envelope: E
}

pub trait EnvelopeContext {
    type Marker;
    type Data;
    fn data(&mut self) -> &mut Self::Data;
}

impl<E> EnvelopeContext for AfterTouchContext<E> {
    type Marker = AfterTouchMarker;
    type Data = E;
    fn data(&mut self) -> &mut Self::Data {
        &mut self.envelope
    }
}
