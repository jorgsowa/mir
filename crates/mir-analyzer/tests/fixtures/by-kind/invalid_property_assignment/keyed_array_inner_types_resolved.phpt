===description===
Types nested inside keyed array shapes in docblocks are resolved through use-aliases.
The resolve_atomic_inner fix adds a TKeyedArray arm so that class-string<T> and other
types inside array{...} properties are properly resolved and no spurious
InvalidPropertyAssignment is raised for a correct assignment.
===config===
suppress=MissingPropertyType,MixedAssignment,UnusedParam
===file===
<?php
namespace App;

class Model {}

class Repository {
    /**
     * @param array{"type": string, "count": int} $opts
     */
    public function query(array $opts): void {}
}

$repo = new Repository();
$repo->query(['type' => 'active', 'count' => 10]);
===expect===
