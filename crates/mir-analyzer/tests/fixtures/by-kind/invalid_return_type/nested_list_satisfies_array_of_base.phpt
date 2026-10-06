===description===
A list nested in an array value satisfies an array of a parent class, at any depth.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {}
class Sub extends Base {}
class Unrelated {}

/**
 * @param array<string, list<Sub>> $l
 * @return array<string, array<int|string, Base>>
 */
function nestedList(array $l): array {
    return $l;
}

/**
 * @param array<string, non-empty-list<Sub>> $l
 * @return array<string, array<int, Base>>
 */
function nestedNonEmptyList(array $l): array {
    return $l;
}

/**
 * @param array<string, non-empty-list<Sub>> $l
 * @return array<string, non-empty-array<int, Base>>
 */
function nonEmptyIntoNonEmpty(array $l): array {
    return $l;
}

/**
 * @param array<string, array<string, array<int, Sub>>> $l
 * @return array<string, array<string, array<int, Base>>>
 */
function deeper(array $l): array {
    return $l;
}

/**
 * @param array<string, list<Base>> $l
 * @return array<string, array<int|string, Sub>>
 */
function parentIntoChild(array $l): array {
    return $l;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Base>>' is not compatible with declared 'array<string, array<int|string, Sub>>'
}

/**
 * @param array<string, list<Unrelated>> $l
 * @return array<string, array<int|string, Base>>
 */
function unrelatedValue(array $l): array {
    return $l;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Unrelated>>' is not compatible with declared 'array<string, array<int|string, Base>>'
}

/**
 * @param array<string, list<Sub>> $l
 * @return array<string, array<string, Base>>
 */
function listIntoStringKeys(array $l): array {
    return $l;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Sub>>' is not compatible with declared 'array<string, array<string, Base>>'
}

/**
 * @param array<string, list<Sub>> $l
 * @return array<string, non-empty-array<int, Base>>
 */
function possiblyEmptyIntoNonEmpty(array $l): array {
    return $l;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array<string, list<Sub>>' is not compatible with declared 'array<string, non-empty-array<int, Base>>'
}
===expect===
