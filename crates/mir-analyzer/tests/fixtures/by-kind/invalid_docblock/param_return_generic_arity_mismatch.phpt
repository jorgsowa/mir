===description===
`@param`/`@return TypedMap<string>` against a class declaring 2 `@template`
params is flagged as an arity mismatch, same as the existing `@var` check
— a bare `TypedMap` (no type args) and a fully correct arg list stay silent.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template K
 * @template V
 */
class TypedMap {}

/** @param TypedMap<string> $m */
function tooFewParamArgs($m): void {}

/** @return TypedMap<string, int, bool> */
function tooManyReturnArgs(): TypedMap { return new TypedMap(); }

/** @param TypedMap $m */
function bareGenericReferenceStaysSilent($m): void {}

/** @param TypedMap<string, int> $m */
function correctArityStaysSilent($m): void {}
===expect===
InvalidDocblock@9:9-9:24: Invalid docblock: TypedMap expects 2 template argument(s), got 1
InvalidDocblock@12:9-12:26: Invalid docblock: TypedMap expects 2 template argument(s), got 3
