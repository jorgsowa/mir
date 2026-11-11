===description===
A literal that is not one of the accepted literals stays an invalid argument, not a coercion.
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

sortBy('up');
//     ^^^^ InvalidArgument: Argument $direction of sortBy() expects '"asc"|"desc"', got '"up"'
sortBy(5);
//     ^ InvalidArgument: Argument $direction of sortBy() expects '"asc"|"desc"', got '5'
sortBy('');
//     ^^ InvalidArgument: Argument $direction of sortBy() expects '"asc"|"desc"', got '""'
