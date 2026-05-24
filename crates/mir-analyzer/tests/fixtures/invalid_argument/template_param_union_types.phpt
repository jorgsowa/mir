===description===
union-like generic types (Result, Either) with template parameters should not report InvalidArgument
===file===
<?php
/** @template S @template E */
class Result { }

/** @template L @template R */
class Either { }

/**
 * @template Success
 * @template Error
 * @param Result<Success, Error> $result
 */
function assertSuccessValue(Result $result): void {}

/**
 * @template LeftType
 * @template RightType
 * @param Either<LeftType, RightType> $either
 */
function processEither(Either $either): void {}

class UserData { }
class ValidationError { }

function test(): void {
    // Result<Success, Error> with concrete types
    /** @var Result<UserData, ValidationError> $result */
    $result = new Result();
    assertSuccessValue($result);

    // Either<L, R> with concrete types
    /** @var Either<string, int> $either */
    $either = new Either();
    processEither($either);

    // Result with union types
    /** @var Result<UserData|null, ValidationError|RuntimeException> $complexResult */
    $complexResult = new Result();
    assertSuccessValue($complexResult);
}
===expect===
UnusedParam@13:29: Parameter $result is never used
UnusedParam@20:24: Parameter $either is never used
