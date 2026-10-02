===description===
Writing a literal key to a bare `array` refines it to an open shape holding that key, so it
satisfies a declared shape; other bases and loops keep their existing behavior.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}

class Holder {
    /** @var array{records: array<int, Foo>} */
    private array $data = ['records' => []];

    /** @param array<int, Foo> $foos */
    public function store(array $foos): void {
        $data = unpack_something();
        $data['records'] = $foos;
        /** @mir-check $data is array{'records': array<int, Foo>, ...} */
        $_ = $data;
        $this->data = $data;
    }
}

function unpack_something(): array { return []; }

/** @param array<int, string> $names */
function more_keys(array $names, bool $flag): void {
    $a = unpack_something();
    $a['x'] = 1;
    $a['y'] = $names;
    /** @mir-check $a is array{'x': 1, 'y': array<int, string>, ...} */
    $_ = $a;

    $a['x'] = 'now a string';
    /** @mir-check $a is array{'x': "now a string", 'y': array<int, string>, ...} */
    $_ = $a;

    $typed = ['k' => 1];
    $typed[$names[0] ?? 'z'] = 2;
    /** @mir-check $typed is array<string, 2|1> */
    $_ = $typed;

    $dynamic = unpack_something();
    $dynamic[$names[0] ?? 'z'] = 2;
    /** @mir-check $dynamic is array<array-key, mixed> */
    $_ = $dynamic;

    $looped = unpack_something();
    foreach ($names as $n) {
        $looped['k'] = $n;
    }
    /** @mir-check $looped is array<array-key, mixed> */
    $_ = $looped;
}
===expect===
