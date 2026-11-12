===description===
`@return new<CONST[T]>` instantiates the class the constant map names for the key the caller passed.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

namespace App;

const DATETIME_CLASSES = [
    'immutable' => \DateTimeImmutable::class,
    'mutable' => \DateTime::class,
];

function takesImmutable(\DateTimeImmutable $value): void {}

function takesMutable(\DateTime $value): void {}

/**
 * @template T of key-of<DATETIME_CLASSES>
 * @param T $type
 * @return new<DATETIME_CLASSES[T]>
 */
function createNow(string $type): \DateTimeInterface
{
    return new (DATETIME_CLASSES[$type]);
}

takesImmutable(createNow('immutable'));
takesMutable(createNow('mutable'));
takesMutable(createNow('immutable'));
//           ^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $value of takesMutable() expects 'DateTime', got 'DateTimeImmutable'
