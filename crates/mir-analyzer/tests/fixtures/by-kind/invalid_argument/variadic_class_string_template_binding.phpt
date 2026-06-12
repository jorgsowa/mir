===description===
Template binds through an aggregate-style variadic docblock param
(Mockery::mock pattern): array<class-string<T>|T|...> ...$args binds T from a
::class argument, and the sibling bare T must not also absorb the
class-string (T would become Conn|class-string<Conn>)
===file===
<?php
interface MockInterface {}

class Mockery {
    /**
     * @template TMock of object
     * @param array<class-string<TMock>|TMock|Closure(MockInterface&TMock):MockInterface&TMock|array<TMock>> $args
     * @return MockInterface&TMock
     */
    public static function mock(...$args) { throw new \Exception(); }
}

class Conn {
    public function query(): string { return ''; }
}

function needsConn(Conn $c): void {}

$mock = Mockery::mock(Conn::class);
/** @mir-check $mock is MockInterface&Conn */
needsConn($mock);
===expect===
