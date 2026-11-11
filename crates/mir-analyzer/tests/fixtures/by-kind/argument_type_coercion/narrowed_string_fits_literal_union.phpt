===description===
A string narrowed to the accepted literals, by comparison, in_array() or array_key_exists(), is not a coercion.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'asc'|'desc' $direction */
function sortBy(string $direction): void {}

function run(string $input): void {
    sortBy('asc');
    if ($input === 'asc' || $input === 'desc') {
        sortBy($input);
    }
    if (in_array($input, ['asc', 'desc'], true)) {
        sortBy($input);
    }
    $labels = ['asc' => 'Ascending', 'desc' => 'Descending'];
    if (array_key_exists($input, $labels)) {
        sortBy($input);
    }
}
