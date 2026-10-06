===description===
An unbounded class-string argument says nothing about the template, so the bound is not checked (function, method, static and interface-string forms).
===file===
<?php
interface Named {}

/**
 * @template T of Named
 * @param class-string<T> $class
 */
function register(string $class): void { echo $class; }

/**
 * @template T of Named
 * @param class-string<T> $class
 * @return T
 */
function make(string $class): object { return new $class(); }

/**
 * @template T of Named
 * @param interface-string<T> $class
 */
function register_interface(string $class): void { echo $class; }

final class Registry {
    /**
     * @template T of Named
     * @param class-string<T> $class
     */
    public function add(string $class): void { echo $class; }

    /**
     * @template T of Named
     * @param class-string<T> $class
     */
    public static function addStatic(string $class): void { echo $class; }
}

/** @param class-string $cls */
function from_class_string(string $cls): void {
    register($cls);
    (new Registry())->add($cls);
    Registry::addStatic($cls);
    register_interface($cls);
}

/** @param interface-string $cls */
function from_interface_string(string $cls): void {
    register_interface($cls);
}

/** @param class-string $cls */
function result_stays_object(string $cls): void {
    $o = make($cls);
    /** @mir-check $o is object */
    echo get_class($o);
}
