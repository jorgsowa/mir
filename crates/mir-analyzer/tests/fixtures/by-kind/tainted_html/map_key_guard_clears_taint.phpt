===description===
`isset($map[$x])` and `array_key_exists($x, $map)` clear `$x` when the map's keys are all known.
A map that may hold other keys proves nothing.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const MAP = ['a' => 1];

class Q {
    const MAP = ['a' => 1, 'b' => 2];

    public function classConstant(): void {
        $x = $_GET['x'];
        if (isset(self::MAP[$x])) {
            echo $x;
        }
    }
}

function local(): void {
    $map = ['a' => 1];
    $x = $_GET['x'];
    if (isset($map[$x])) {
        echo $x;
    }
}

function keyExists(): void {
    $x = $_GET['x'];
    if (!array_key_exists($x, MAP)) {
        exit;
    }
    echo $x;
}

function element(): void {
    if (isset(MAP[$_GET['x']])) {
        echo $_GET['x'];
    }
}

function failingPath(): void {
    $x = $_GET['x'];
    if (!isset(MAP[$x])) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function mapGrown(): void {
    $map = ['a' => 1];
    $map[$_GET['k']] = 2;
    $x = $_GET['x'];
    if (isset($map[$x])) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}

function openMap(array $map): void {
    $x = $_GET['x'];
    if (isset($map[$x])) {
        echo $x;
//      ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
