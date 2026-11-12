===description===
`array_keys($a['k'])` carries the taint of the keys written under `$a['k']`, not of the values stored there, including for an array a method builds and returns. A tainted key, or a whole tainted array stored under `'k'`, is still reported.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class ModuleList
{
    private function build(array $values): array
    {
        $modules = ['install' => []];
        foreach (['node', 'user'] as $name) {
            $modules['install'][$name] = $values[$name];
        }
        return $modules;
    }

    public function submit(): void
    {
        $modules = $this->build($_POST);
        foreach (array_keys($modules['install']) as $module) {
            $module();
        }
    }
}

function taintedValue(string $name): void
{
    $modules = ['install' => []];
    $modules['install'][$name] = $_GET['label'];
    foreach (array_keys($modules['install']) as $module) {
        $module();
    }
}

function taintedKey(): void
{
    $modules = ['install' => []];
    $modules['install'][$_GET['name']] = 'label';
    foreach (array_keys($modules['install']) as $module) {
        $module();
//      ^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}

function taintedArray(): void
{
    $modules = ['install' => []];
    $modules['install'] = $_GET;
    foreach (array_keys($modules['install']) as $module) {
        $module();
//      ^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    }
}
