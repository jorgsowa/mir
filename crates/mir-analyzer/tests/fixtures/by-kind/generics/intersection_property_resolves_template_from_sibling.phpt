===description===
Reading a templated property through `C<string>&B`, where the bare `B`
implements `C<T>`, binds `B`'s template from the `C<string>` part.
===file===
<?php
/** @template T */
interface Container {
    /** @return T */
    public function get(): mixed;
}

/**
 * @template T
 * @implements Container<T>
 */
final class Box implements Container {
    /** @param T $value */
    public function __construct(public mixed $value) {}

    public function get(): mixed {
        return $this->value;
    }
}

/** @param Container<string>&Box $v */
function bareClassPart(Container&Box $v): int {
    /** @mir-check $v->value is string */
    echo 1;
    return strlen($v->value);
}

/** @param Container<string>&Box<int> $v */
function ownTypeArgsWin(Container&Box $v): int {
    /** @mir-check $v->value is int */
    echo 1;
    return $v->value;
}
