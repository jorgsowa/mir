===description===
Promoted properties expand class @psalm-type aliases from their own @var or the ctor @param, including when read from another file
===config===
suppress=UnusedVariable,UnusedParam
===file:holder.php===
<?php
namespace App;

/**
 * @psalm-type Conf = array{name: string, port: int}
 * @template T
 */
final class Holder {
    /** @param Conf $viaParam */
    public function __construct(
        public array $viaParam,
        /** @var Conf */
        public array $viaVar,
        /** @var list<Conf> */
        public array $many,
        /** @var Conf */
        private array $hidden,
        /** @var T */
        public mixed $generic,
    ) {}

    public function port(): int {
        $c = $this->hidden;
        /** @mir-check $c is array{name: string, port: int} */
        echo 1;
        return $this->viaVar['port'];
    }
}
===file:app.php===
<?php
namespace App;

/** @param Holder<string> $h */
function read(Holder $h): void {
    $a = $h->viaParam;
    /** @mir-check $a is array{name: string, port: int} */
    echo 1;
    $b = $h->viaVar;
    /** @mir-check $b is array{name: string, port: int} */
    echo 1;
    $c = $h->many;
    /** @mir-check $c is list<array{name: string, port: int}> */
    echo 1;
    $d = $h->generic;
    /** @mir-check $d is string */
    echo 1;
    echo $h->viaVar['port'] + 1;
}
===expect===
