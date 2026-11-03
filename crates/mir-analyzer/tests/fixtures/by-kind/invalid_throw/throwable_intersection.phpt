===description===
An intersection with a `Throwable` part can be thrown, including an anonymous
exception class that implements a marker interface.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface CacheException {}
interface Marker {}

function anonymous(float $beta): void {
    if ($beta < 0) {
        throw new class('negative') extends \InvalidArgumentException implements CacheException {};
    }
}

function intersection(\RuntimeException&CacheException $e): void {
    throw $e;
}

function notThrowable(\ArrayObject&Marker $o): void {
    throw $o;
//  ^^^^^^^^^ InvalidThrow: Thrown type 'ArrayObject&Marker' does not extend Throwable
}
