===description===
An intersection return is satisfied when every part is covered by the value's class hierarchy
===file===
<?php
interface Readable {}
interface Writable {}
interface Closable {}
abstract class Stream {}
abstract class Channel extends Stream implements Readable, Writable {}

final class Holder {
    public function __construct(private Channel&Writable $channel, private Channel $plain) {}

    public function intersectionToIntersection(): Readable&Stream {
        return $this->channel;
    }

    public function intersectionToOverlappingIntersection(): Readable&Writable {
        return $this->channel;
    }

    public function intersectionToPartClass(): Stream {
        return $this->channel;
    }

    public function plainToIntersection(): Readable&Stream {
        return $this->plain;
    }

    public function plainToNullableIntersection(): (Readable&Stream)|null {
        return $this->plain;
    }

    public function intersectionToUncoveredPart(): Readable&Closable {
        return $this->channel;
//      ^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Channel&Writable' is not compatible with declared 'Readable&Closable'
    }

    public function plainToUncoveredPart(): Stream&Closable {
        return $this->plain;
//      ^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'Channel' is not compatible with declared 'Stream&Closable'
    }
}

/** @param Channel&Writable $c @return Readable&Stream */
function viaDocblock($c) {
    /** @mir-check $c is Channel&Writable */
    return $c;
}
