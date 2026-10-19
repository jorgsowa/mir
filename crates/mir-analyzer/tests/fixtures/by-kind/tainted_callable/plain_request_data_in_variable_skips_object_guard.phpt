===description===
A callee that only calls what it proves is a `Closure` (`instanceof`) is not
reached by request data held in a variable: strings and arrays of strings are
never closures. Once the variable may hold something computed (a decoded
object, a branch or element write, a reference) the flow is reported again.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Runner {
    public function each(array $items): void {
        foreach ($items as $item) {
            if ($item instanceof \Closure) {
                $item(1);
            }
        }
    }

    public function first(array $items): void {
        $item = reset($items);
        if ($item instanceof \Closure) {
            $item(1);
        }
    }

    public function shifted(array $items): void {
        $item = array_shift($items);
        if (!$item instanceof \Closure) {
            return;
        }
        $item(1);
    }

    public function direct(mixed $item): void {
        if ($item instanceof \Closure) {
            $item(1);
        }
    }

    public function anything(mixed $item): void {
        if (is_callable($item)) {
            $item(1);
        }
    }
}

function plain(Runner $runner): void {
    $query = ['k' => $_GET['t']];
    $runner->each($query);
    $token = $_GET['t'];
    $runner->each([$token]);
    $runner->direct($token);
    $copy = $query;
    $runner->each($copy);
    $runner->first($copy);
    $runner->shifted($copy);
    $nested = ['inner' => $query];
    $runner->each($nested);
    $runner->each($_GET);
}

function branches(Runner $runner, bool $flag): void {
    if ($flag) {
        $data = $_GET['a'];
    } else {
        $data = ['b' => $_POST['b']];
    }
    $runner->each([$data]);
}

function computed(Runner $runner, bool $flag): void {
    $decoded = json_decode((string) $_GET['j']);
    $runner->direct($decoded);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $data = $_GET['t'];
    $data = json_decode($data);
    $runner->direct($data);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $bag = $_GET;
    $bag['o'] = json_decode((string) $_GET['j']);
    $runner->each($bag);
//  ^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $list = ['x' => $_GET['t']];
    $list[] = json_decode((string) $_GET['j']);
    $runner->each($list);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $maybe = $_GET['t'];
    if ($flag) {
        $maybe = json_decode((string) $maybe);
    }
    $runner->direct($maybe);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $plain = $_GET['t'];
    $alias = &$plain;
    $alias = json_decode((string) $alias);
    $runner->direct($plain);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs

    $callable = $_GET['t'];
    $runner->anything($callable);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}
