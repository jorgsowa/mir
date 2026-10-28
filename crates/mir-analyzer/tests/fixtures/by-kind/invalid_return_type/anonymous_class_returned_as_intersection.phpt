===description===
An anonymous class (typed `object`) returned as `A&B` is less specific, as it is for a single class
===file===
<?php
namespace App;

interface Handler {}
interface Named {}

function both(): Handler&Named {
    return new class implements Handler, Named {};
}

function builtin(): \Traversable&\Stringable {
    return new class implements \IteratorAggregate, \Stringable {
        public function getIterator(): \Traversable {
            yield 'a';
        }

        public function __toString(): string {
            return '';
        }
    };
}

function notAnObject(): Handler&Named {
    return 1;
//  ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'App\Handler&App\Named'
}
