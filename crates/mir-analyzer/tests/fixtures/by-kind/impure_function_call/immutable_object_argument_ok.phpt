===description===
An object whose class is immutable cannot be mutated by the callee, so passing
it from an immutable or external-mutation-free context into an impure
function, constructor, method or static method is allowed. A mutable object
argument stays flagged.
===config===
<mir>
  <issueHandlers>
    <UnusedClass errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Buffer {
    public string $data = '';
}

final class Sink {
    public function __construct(public object $value) {}

    public function take(object $value): void {}

    public static function store(object $value): void {}
}

/** @psalm-immutable */
final class Token implements JsonSerializable {
    public function __construct(private string $value, private Buffer $buffer) {}

    public function jsonSerialize(): string {
        return $this->value;
    }

    public function encoded(): string {
        $json = json_encode($this);
        /** @mir-check $json is string|false */
        return (string) $json;
    }

    public function wrapped(): Sink {
        return new Sink($this);
    }

    public function handOver(Sink $sink): void {
        $sink->take($this);
        Sink::store($this);
    }

    public function leakBuffer(): string {
        return (string) json_encode($this->buffer);
//                      ^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function json_encode() in a @pure function
    }
}

final class Encoder {
    /** @psalm-external-mutation-free */
    public function encode(Token $token, Buffer $buffer): Sink {
        Sink::store($token);
        new Sink($buffer);
//      ^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function Sink::__construct() in a @pure function
        return new Sink($token);
    }
}
