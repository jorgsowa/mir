===description===
`isset($a['k']->prop)` proves `$a['k']` is present and non-null, so reads of
that key inside the guard are not reported. Outside it the key is only possibly absent.
===file===
<?php
final class Inner {
    public string $name = '';
}

final class Config {
    public string $name = '';
    public Inner $child;

    public function __construct() {
        $this->child = new Inner();
    }
}

function plain(bool $f): void {
    $d = [];
    if ($f) {
        $d['cfg'] = new Config();
    }
    if (isset($d['cfg']->name)) {
        /** @mir-check $d['cfg'] is Config */
        $_ = $d['cfg'];
        echo $d['cfg']->name;
    }
}

function nullsafe(bool $f): void {
    $d = [];
    if ($f) {
        $d['cfg'] = new Config();
    }
    if (isset($d['cfg']?->name)) {
        echo $d['cfg']->name;
    }
}

function chained(bool $f): void {
    $d = [];
    if ($f) {
        $d['cfg'] = new Config();
    }
    if (isset($d['cfg']->child->name)) {
        echo $d['cfg']->child->name;
    }
}

function nestedKeys(bool $f): void {
    $d = ['outer' => []];
    if ($f) {
        $d['outer']['cfg'] = new Config();
    }
    if (isset($d['outer']['cfg']->name)) {
        echo $d['outer']['cfg']->name;
    }
}

function conjunction(bool $f): void {
    $d = [];
    if ($f) {
        $d['cfg'] = new Config();
    }
    if (isset($d['cfg']->name) && $d['cfg']->name !== '') {
        echo $d['cfg']->name;
    }
}

function unguarded(bool $f): void {
    $d = [];
    if ($f) {
        $d['cfg'] = new Config();
    }
    echo $d['cfg']->name;
}
===expect===
