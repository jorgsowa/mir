===description===
Sibling of nested_array_shape_value_no_error: a genuinely wrong inner shape value still errors.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @return array<string, array{id: int, name: string}>
 */
function getItems(): array {
    return ['a' => ['id' => 'not-an-int', 'name' => 'x']];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{'a': array{'id': "not-an-int", 'name': "x"}}' is not compatible with declared 'array<string, array{'id': int, 'name': string}>'
}
