===description===
A call to a method that stores its argument in a static property
(`self::$w = $s`) leaves that static property tainted for the reads that follow
in the calling body. Sanitized or clean arguments and later overwrites do not.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Registry {
    public static $w = '';
    public static $other = '';

    public static function put(string $s): void {
        self::$w = $s;
    }

    public static function putLate(string $s): void {
        static::$w = $s;
    }

    public static function putClean(string $s): void {
        self::$w = htmlspecialchars($s);
    }

    public function remember(string $s): void {
        self::$w = $s;
    }

    public static function putOther(string $s): void {
        self::$other = $s;
    }

    public static function viaSelf(string $s): void {
        self::put($s);
        echo self::$w;
    }

    public static function viaOwnClean(): void {
        self::put('literal');
        echo self::$w;
    }
}

function readInto(&$out) { $out = $_GET['a']; }
//                 ^^^^ MissingParamType: Parameter $out of readInto() has no type annotation

class Child extends Registry {
    public static function viaChild(string $s): void {
        parent::put($s);
        echo parent::$w;
    }
}

function direct() {
    Registry::put($_GET['x']);
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function late() {
    Registry::putLate($_GET['x']);
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function sanitizedArgument() {
    Registry::put(htmlspecialchars($_GET['x']));
    echo Registry::$w;
}

function sanitizingMethod() {
    Registry::putClean($_GET['x']);
    echo Registry::$w;
}

function cleanArgument() {
    Registry::put('literal');
    echo Registry::$w;
}

function overwritten() {
    Registry::put($_GET['x']);
    Registry::$w = 'clean';
    echo Registry::$w;
}

function otherProperty() {
    Registry::put($_GET['x']);
    echo Registry::$other;
}

function instanceMethod() {
    $r = new Registry();
    $r->remember($_GET['x']);
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function inheritedCall() {
    Child::put($_GET['x']);
    echo Child::$w;
//  ^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function otherProperty2() {
    Registry::putOther($_GET['x']);
    echo Registry::$other;
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function taintedLocal() {
    $q = $_GET['x'];
    Registry::put($q);
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function throughWrapper() {
    Registry::viaSelf($_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function byRefArgument() {
    readInto(Registry::$w);
    echo Registry::$w;
//  ^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function byRefArgumentOtherProperty() {
    readInto(Registry::$w);
    echo Registry::$other;
}
