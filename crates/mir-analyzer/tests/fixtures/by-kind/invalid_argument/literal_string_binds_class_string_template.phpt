===description===
A class-name-shaped string literal coerces to class-string when binding a
class-string<T> template (m::mock('Foo\Bar') with a plain string instead of
::class). Non-class-shaped strings ('alias:Foo') don't bind.
===file===
<?php
namespace Acme {
    class Conn {}
}
namespace {
    interface MockInterface {}
    class Mockery {
        /**
         * @template TMock of object
         * @param array<class-string<TMock>|TMock> $args
         * @return MockInterface&TMock
         */
        public static function mock(...$args) { throw new \Exception(); }
    }
    function needsConn(Acme\Conn $c): void {}

    $mock = Mockery::mock('Acme\Conn');
    /** @mir-check $mock is MockInterface&Acme\Conn */
    needsConn($mock);
}
===expect===
