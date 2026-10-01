<?php

declare(strict_types=1);

/**
 * mir Psalm-plugin host.
 *
 * Long-lived subprocess spawned by mir (crates/mir-plugin/src/psalm/mod.rs).
 * Speaks JSON-lines RPC on stdin/stdout:
 *   -> {"id":1,"method":"init","params":{"projectRoot":"...","plugins":[{"class":"...","configXml":null}]}}
 *   <- {"id":1,"result":{"stubs":[...],"functionIds":[...],"methodClasses":[...],"warnings":[...]}}
 *   -> {"id":2,"method":"functionReturnType","params":{"functionId":"...","argTypes":[...],"snippet":"...","file":"..."}}
 *   <- {"id":2,"result":{"type":"list<int>"}}
 *
 * Psalm plugin classes and Psalm itself are loaded from the analyzed
 * project's own vendor/autoload.php, plus a separately installed Psalm
 * (explicit "psalmAutoload" param, or discovered under the project root) when
 * the project's autoloader does not provide it. Shim implementations of Psalm's
 * interfaces are GENERATED at runtime from the installed Psalm's actual
 * interface signatures (see MirShimGenerator), so this host does not break
 * when Psalm adds or changes interface methods.
 */

ini_set('display_errors', 'stderr');
// Psalm's scanner holds every scanned class in memory.
ini_set('memory_limit', '-1');

final class MirShimGenerator
{
    /**
     * Generate and eval a class implementing $interface. $specialBodies maps
     * method name (lowercase) to a PHP body; every other method gets a
     * neutral default derived from its return type. Bodies must use
     * func_get_args() — parameter names differ across Psalm versions.
     */
    public static function implement(string $interface, string $className, array $specialBodies): string
    {
        if (class_exists($className, false)) {
            return $className;
        }
        $rc = new ReflectionClass($interface);
        $methods = [];
        foreach ($rc->getMethods() as $m) {
            if ($m->isStatic()) {
                continue;
            }
            $body = $specialBodies[strtolower($m->getName())]
                ?? self::defaultBody($m->getReturnType(), $m->getName());
            $methods[] = '    public function ' . $m->getName()
                . '(' . self::renderParams($m) . ')'
                . self::renderReturnType($m) . " {\n        $body\n    }";
        }
        $code = 'class ' . $className . ' implements \\' . $interface . " {\n"
            . implode("\n", $methods) . "\n}";
        eval($code);
        return $className;
    }

    private static function renderParams(ReflectionMethod $m): string
    {
        $out = [];
        foreach ($m->getParameters() as $p) {
            $s = '';
            if ($t = $p->getType()) {
                $s .= self::renderType($t) . ' ';
            }
            if ($p->isPassedByReference()) {
                $s .= '&';
            }
            if ($p->isVariadic()) {
                $s .= '...';
            }
            $s .= '$' . $p->getName();
            if ($p->isDefaultValueAvailable()) {
                $s .= ' = ' . var_export($p->getDefaultValue(), true);
            } elseif ($p->allowsNull() && $t && !$p->isVariadic() && !($t instanceof ReflectionNamedType && $t->getName() === 'mixed')) {
                // keep optionalness lax; interfaces may add params later
            }
            $out[] = $s;
        }
        return implode(', ', $out);
    }

    private static function renderReturnType(ReflectionMethod $m): string
    {
        $t = $m->getReturnType();
        return $t === null ? '' : (': ' . self::renderType($t));
    }

    private static function renderType(ReflectionType $t): string
    {
        if ($t instanceof ReflectionUnionType) {
            return implode('|', array_map([self::class, 'renderType'], $t->getTypes()));
        }
        if ($t instanceof ReflectionIntersectionType) {
            return implode('&', array_map([self::class, 'renderType'], $t->getTypes()));
        }
        /** @var ReflectionNamedType $t */
        $name = $t->getName();
        $rendered = $t->isBuiltin() ? $name : '\\' . $name;
        if ($t->allowsNull() && $name !== 'mixed' && $name !== 'null') {
            $rendered = '?' . $rendered;
        }
        return $rendered;
    }

    private static function defaultBody(?ReflectionType $t, string $methodName): string
    {
        if ($t === null) {
            return 'return null;';
        }
        if ($t instanceof ReflectionNamedType) {
            if ($t->allowsNull()) {
                return $t->getName() === 'void' ? 'return;' : 'return null;';
            }
            switch ($t->getName()) {
                case 'void':
                    return 'return;';
                case 'bool':
                    return 'return false;';
                case 'int':
                    return 'return 0;';
                case 'float':
                    return 'return 0.0;';
                case 'string':
                    return "return '';";
                case 'array':
                case 'iterable':
                    return 'return [];';
                case 'mixed':
                    return 'return null;';
                case 'static':
                case 'self':
                    return 'return $this;';
            }
        }
        if ($t instanceof ReflectionUnionType) {
            if ($t->allowsNull()) {
                return 'return null;';
            }
            foreach ($t->getTypes() as $member) {
                if ($member instanceof ReflectionNamedType && $member->isBuiltin()) {
                    return self::defaultBody($member, $methodName);
                }
            }
        }
        return 'throw new \\BadMethodCallException('
            . var_export("$methodName() is not supported by the mir psalm bridge", true) . ');';
    }
}

final class MirPsalmHost
{
    public static ?MirPsalmHost $instance = null;

    /** @var list<string> stub file paths registered by plugins */
    public array $stubs = [];
    /** @var array<string, class-string> lowercase function id -> provider class */
    public array $functionProviders = [];
    /** @var array<string, class-string> lowercase FQCN -> provider class */
    public array $methodProviders = [];
    /** @var list<class-string> */
    public array $afterFunctionLikeHandlers = [];
    /** @var list<string> */
    public array $warnings = [];

    public string $currentFile = '';
    public ?string $callingClass = null;
    public ?object $currentAliases = null;
    /** @var array<class-string, true> */
    private array $classLikeHandlers = [];
    /** @var list<class-string> */
    private array $nativeHandlers = [];
    /** @var array<string, true> */
    private array $scannedFiles = [];
    /** @var array<string, array> */
    private array $parsedFiles = [];
    private ?object $composerLoader = null;
    private bool $codebaseChanged = false;
    private int $currentSpanEnd = 0;
    private string $projectRoot = '';
    private ?\Psalm\Internal\Analyzer\ProjectAnalyzer $projectAnalyzer = null;
    /** @var array<string, string> */
    private array $fileContents = [];
    private ?object $lastCallNode = null;
    /** @var array<string, string> */
    private array $knownFiles = [];
    public SplObjectStorage $nodeTypes;
    public ?object $nodeTypeProvider = null;
    private ?object $statementsSource = null;
    private $phpParser = null;

    public function __construct()
    {
        $this->nodeTypes = new SplObjectStorage();
        self::$instance = $this;
    }

    // -- RPC dispatch --------------------------------------------------------

    public function dispatch(string $method, array $params): array
    {
        switch ($method) {
            case 'init':
                return $this->init($params);
            case 'functionReturnType':
                return $this->functionReturnType($params);
            case 'methodReturnType':
                return $this->methodReturnType($params);
            case 'afterFunctionLike':
                return $this->afterFunctionLike($params);
            case 'afterClassLike':
                return $this->afterClassLike($params);
            case 'shutdown':
                return [];
            default:
                throw new RuntimeException("unknown method: $method");
        }
    }

    private function init(array $params): array
    {
        $root = (string)($params['projectRoot'] ?? getcwd());
        $this->projectRoot = $root;
        $autoload = $params['autoload'] ?? ($root . '/vendor/autoload.php');
        if (is_file($autoload)) {
            $loader = require $autoload;
            $this->composerLoader = is_object($loader) && method_exists($loader, 'findFile') ? $loader : null;
        }

        if (!interface_exists('Psalm\\Plugin\\PluginEntryPointInterface')) {
            $explicit = $params['psalmAutoload'] ?? null;
            $psalmAutoload = $explicit !== null ? (string)$explicit : self::findPsalmAutoload($root);
            if ($psalmAutoload !== null && is_file($psalmAutoload)) {
                require $psalmAutoload;
            }
        }

        if (!interface_exists('Psalm\\Plugin\\PluginEntryPointInterface')) {
            throw new RuntimeException(
                "Psalm (vimeo/psalm) was not found: it is not in $autoload and no separate install was "
                . 'found under the project root. Install it, or point MIR_PSALM_AUTOLOAD at the '
                . "autoload.php of the install that provides it."
            );
        }

        $registrationClass = MirShimGenerator::implement(
            'Psalm\\Plugin\\RegistrationInterface',
            'MirRegistrationShim',
            [
                'addstubfile' => '\\MirPsalmHost::$instance->stubs[] = (string)(func_get_args()[0] ?? \'\');',
                'registerpreloadedstubfile' => '\\MirPsalmHost::$instance->stubs[] = (string)(func_get_args()[0] ?? \'\');',
                'registerhooksfromclass' => '\\MirPsalmHost::$instance->registerHooks((string)(func_get_args()[0] ?? \'\'));',
            ]
        );
        $registration = new $registrationClass();

        foreach (($params['plugins'] ?? []) as $spec) {
            $class = (string)($spec['class'] ?? '');
            if ($class === '') {
                continue;
            }
            if (!class_exists($class)) {
                $this->warnings[] = "plugin class $class not found via the project autoloader — skipped";
                continue;
            }
            $config = null;
            if (!empty($spec['configXml'])) {
                try {
                    $config = new SimpleXMLElement((string)$spec['configXml']);
                } catch (Throwable $e) {
                    $this->warnings[] = "plugin $class: invalid config XML: {$e->getMessage()}";
                }
            }
            try {
                $entry = new $class();
                $entry($registration, $config);
            } catch (Throwable $e) {
                $this->warnings[] = "plugin $class entry point failed: {$e->getMessage()} — skipped";
            }
        }

        foreach (($params['pluginFiles'] ?? []) as $file) {
            $this->loadPluginFile((string)$file);
        }

        return [
            'stubs' => array_values(array_unique($this->stubs)),
            'functionIds' => array_keys($this->functionProviders),
            'methodClasses' => array_keys($this->methodProviders),
            'afterFunctionLike' => $this->afterFunctionLikeHandlers !== [],
            'afterClassLike' => $this->classLikeHandlers !== [],
            'warnings' => $this->warnings,
        ];
    }

    /**
     * Locate a Psalm install living outside the project's own vendor dir
     * (e.g. an isolated tools install). Breadth-first so the shallowest wins.
     */
    private static function findPsalmAutoload(string $root): ?string
    {
        $queue = [[$root, 0]];
        $visited = 0;
        while ($queue && $visited++ < 2000) {
            [$dir, $depth] = array_shift($queue);
            foreach ([$dir, $dir . '/vendor'] as $vendorDir) {
                if (is_file($vendorDir . '/autoload.php') && is_dir($vendorDir . '/vimeo/psalm')) {
                    return $vendorDir . '/autoload.php';
                }
            }
            if ($depth >= 3) {
                continue;
            }
            $children = @scandir($dir) ?: [];
            foreach ($children as $child) {
                if ($child[0] === '.' || $child === 'vendor' || $child === 'node_modules') {
                    continue;
                }
                if (is_dir($dir . '/' . $child)) {
                    $queue[] = [$dir . '/' . $child, $depth + 1];
                }
            }
        }
        return null;
    }

    private function loadPluginFile(string $file): void
    {
        if (!is_file($file)) {
            $this->warnings[] = "plugin file $file not found — skipped";
            return;
        }
        try {
            $class = self::firstClassInFile($file);
            require_once $file;
            if ($class === null || !class_exists($class)) {
                $this->warnings[] = "plugin file $file declares no class — skipped";
                return;
            }
            $this->registerHooks($class);
        } catch (Throwable $e) {
            $this->warnings[] = "plugin file $file failed to load: {$e->getMessage()} — skipped";
        }
    }

    /** First class declared in the file, matching how Psalm picks a file plugin's hook class. */
    private static function firstClassInFile(string $file): ?string
    {
        $namespace = '';
        $tokens = PhpToken::tokenize((string)file_get_contents($file));
        $count = count($tokens);
        for ($i = 0; $i < $count; $i++) {
            $token = $tokens[$i];
            if ($token->is(T_NAMESPACE)) {
                $namespace = '';
                for ($j = $i + 1; $j < $count && !$tokens[$j]->is(['{', ';']); $j++) {
                    if (!$tokens[$j]->isIgnorable()) {
                        $namespace .= $tokens[$j]->text;
                    }
                }
            } elseif ($token->is(T_CLASS)) {
                $prev = $i > 0 ? $tokens[$i - 1] : null;
                if ($prev !== null && $prev->is([T_DOUBLE_COLON, T_NEW])) {
                    continue;
                }
                for ($j = $i + 1; $j < $count; $j++) {
                    if ($tokens[$j]->is(T_STRING)) {
                        return ($namespace !== '' ? $namespace . '\\' : '') . $tokens[$j]->text;
                    }
                }
            }
        }
        return null;
    }

    public function registerHooks(string $class): void
    {
        if (!class_exists($class)) {
            $this->warnings[] = "hook class $class not found — skipped";
            return;
        }
        $interfaces = class_implements($class) ?: [];
        $used = false;
        foreach ($interfaces as $iface) {
            if (str_ends_with($iface, 'EventHandler\\FunctionReturnTypeProviderInterface')) {
                foreach ($class::getFunctionIds() as $id) {
                    $this->functionProviders[strtolower(ltrim((string)$id, '\\'))] = $class;
                }
                $used = true;
            } elseif (str_ends_with($iface, 'EventHandler\\MethodReturnTypeProviderInterface')) {
                foreach ($class::getClassLikeNames() as $fqcn) {
                    $this->methodProviders[strtolower(ltrim((string)$fqcn, '\\'))] = $class;
                }
                $used = true;
            } elseif (
                str_ends_with($iface, 'EventHandler\\AfterClassLikeAnalysisInterface')
                || str_ends_with($iface, 'EventHandler\\AfterClassLikeVisitInterface')
                || str_ends_with($iface, 'EventHandler\\AfterCodebasePopulatedInterface')
            ) {
                if (!isset($this->classLikeHandlers[$class])) {
                    $this->classLikeHandlers[$class] = true;
                    $this->nativeHandlers[] = $class;
                }
                $used = true;
            } elseif (str_ends_with($iface, 'EventHandler\\AfterFunctionLikeAnalysisInterface')) {
                $this->afterFunctionLikeHandlers[] = $class;
                $used = true;
            } elseif (strpos($iface, 'Psalm\\Plugin\\EventHandler\\') === 0) {
                $short = substr($iface, strrpos($iface, '\\') + 1);
                $this->warnings[] = "$class registers $short — not supported by the mir psalm bridge yet, skipped";
            }
        }
        if (!$used && !$interfaces) {
            $this->warnings[] = "$class implements no recognized Psalm event-handler interfaces";
        }
    }

    // -- Return-type providers ------------------------------------------------

    private function functionReturnType(array $params): array
    {
        $id = strtolower(ltrim((string)($params['functionId'] ?? ''), '\\'));
        $class = $this->functionProviders[$id] ?? null;
        if ($class === null) {
            return ['type' => null];
        }
        try {
            $event = $this->buildEvent(
                'Psalm\\Plugin\\EventHandler\\Event\\FunctionReturnTypeProviderEvent',
                $params,
                ['function_id' => $id]
            );
            $union = $class::getFunctionReturnType($event);
            return ['type' => $union === null ? null : (string)$union, 'issues' => $this->takeIssues()];
        } catch (Throwable $e) {
            $this->takeIssues();
            $this->warnOnce("function provider $class failed for $id: {$e->getMessage()}");
            return ['type' => null];
        }
    }

    private function methodReturnType(array $params): array
    {
        $fqcn = ltrim((string)($params['fqcn'] ?? ''), '\\');
        $method = strtolower((string)($params['methodName'] ?? ''));
        $class = $this->methodProviders[strtolower($fqcn)] ?? null;
        if ($class === null) {
            return ['type' => null];
        }
        try {
            $event = $this->buildEvent(
                'Psalm\\Plugin\\EventHandler\\Event\\MethodReturnTypeProviderEvent',
                $params,
                [
                    'fq_classlike_name' => $fqcn,
                    'method_name_lowercase' => $method,
                    'called_fq_classlike_name' => $fqcn,
                    'called_method_name_lowercase' => $method,
                ]
            );
            $union = $class::getMethodReturnType($event);
            return ['type' => $union === null ? null : (string)$union, 'issues' => $this->takeIssues()];
        } catch (Throwable $e) {
            $this->takeIssues();
            $this->warnOnce("method provider $class failed for $fqcn::$method: {$e->getMessage()}");
            return ['type' => null];
        }
    }

    /**
     * Construct a Psalm event by matching its constructor parameter NAMES
     * against what we can supply — resilient to parameter reordering and
     * additions across Psalm versions.
     */
    private function buildEvent(string $eventClass, array $params, array $extra): object
    {
        if (!class_exists($eventClass)) {
            throw new RuntimeException("$eventClass does not exist in this Psalm version");
        }
        $this->currentFile = $this->absolutePath((string)($params['file'] ?? 'unknown.php'));
        $this->currentSpanEnd = (int)($params['spanEnd'] ?? 0);
        $class = $params['callingClass'] ?? $params['class'] ?? null;
        $this->callingClass = $class !== null ? (string)$class : null;
        $snippet = isset($params['snippet']) ? (string)$params['snippet'] : null;
        $argTypes = array_map('strval', (array)($params['argTypes'] ?? []));
        $spanStart = (int)($params['spanStart'] ?? 0);
        $spanEnd = (int)($params['spanEnd'] ?? $spanStart);

        $this->psalmCodebase();
        $source = $this->statementsSource();
        $callArgs = $this->buildCallArgs($snippet, $argTypes, $spanStart);
        $context = new \Psalm\Context();
        $location = $this->rawLocation($spanStart, $spanEnd);

        $byName = $extra + [
            'statements_source' => $source,
            'statements_analyzer' => $source,
            'source' => $source,
            'call_args' => $callArgs,
            'function_args' => $callArgs,
            'stmt' => $this->lastCallNode,
            'context' => $context,
            'code_location' => $location,
            'template_type_parameters' => null,
        ];

        $rc = new ReflectionClass($eventClass);
        $ctor = $rc->getConstructor();
        $args = [];
        foreach ($ctor ? $ctor->getParameters() : [] as $p) {
            if (array_key_exists($p->getName(), $byName)) {
                $args[] = $byName[$p->getName()];
            } elseif ($p->isDefaultValueAvailable()) {
                $args[] = $p->getDefaultValue();
            } elseif ($p->allowsNull()) {
                $args[] = null;
            } else {
                throw new RuntimeException("cannot supply constructor parameter \${$p->getName()} of $eventClass");
            }
        }
        return $rc->newInstanceArgs($args);
    }

    /**
     * Build PhpParser Arg nodes for the call. Primary path re-parses the real
     * call snippet so literal arguments survive; fallback synthesizes
     * variables. Each arg value node gets its mir-inferred type registered in
     * the NodeTypeProvider shim.
     */
    private function buildCallArgs(?string $snippet, array $argTypes, int $spanStart = 0): array
    {
        $args = null;
        $this->lastCallNode = null;
        if ($snippet !== null && $snippet !== '') {
            try {
                $stmts = $this->parser()->parse("<?php\n" . $snippet . ';');
                $finder = new \PhpParser\NodeFinder();
                $call = $finder->findFirst($stmts ?? [], static function ($node): bool {
                    return $node instanceof \PhpParser\Node\Expr\FuncCall
                        || $node instanceof \PhpParser\Node\Expr\MethodCall
                        || $node instanceof \PhpParser\Node\Expr\StaticCall
                        || $node instanceof \PhpParser\Node\Expr\NullsafeMethodCall
                        || $node instanceof \PhpParser\Node\Expr\New_;
                });
                if ($call !== null) {
                    $candidate = $call->getArgs();
                    if (count($candidate) === count($argTypes)) {
                        $args = $candidate;
                        self::shiftPositions([$call], $spanStart - strlen("<?php\n"));
                        $this->lastCallNode = $call;
                    }
                }
            } catch (Throwable $e) {
                // fall through to synthetic args
            }
        }
        if ($args === null) {
            $args = [];
            foreach (array_keys($argTypes) as $i) {
                $args[] = new \PhpParser\Node\Arg(new \PhpParser\Node\Expr\Variable("__mir_arg$i"));
            }
            $this->lastCallNode = new \PhpParser\Node\Expr\MethodCall(
                new \PhpParser\Node\Expr\Variable('__mir_receiver'),
                new \PhpParser\Node\Identifier('__mir_method'),
                $args
            );
        }

        $this->nodeTypes = new SplObjectStorage();
        foreach ($args as $i => $arg) {
            $typeString = $argTypes[$i] ?? 'mixed';
            try {
                $union = \Psalm\Type::parseString($typeString);
            } catch (Throwable $e) {
                $union = \Psalm\Type::getMixed();
            }
            $this->nodeTypes[$arg->value] = $union;
        }
        return $args;
    }

    private function parser()
    {
        if ($this->phpParser === null) {
            $factory = new \PhpParser\ParserFactory();
            $this->phpParser = method_exists($factory, 'createForHostVersion')
                ? $factory->createForHostVersion()
                : $factory->create(\PhpParser\ParserFactory::PREFER_PHP7);
        }
        return $this->phpParser;
    }

    private function statementsSource(): object
    {
        if ($this->statementsSource === null) {
            $providerClass = MirShimGenerator::implement(
                'Psalm\\NodeTypeProvider',
                'MirNodeTypeProviderShim',
                [
                    'gettype' => '$n = func_get_args()[0] ?? null; $s = \\MirPsalmHost::$instance->nodeTypes;'
                        . ' return ($n !== null && isset($s[$n])) ? $s[$n] : null;',
                    'settype' => '$a = func_get_args(); if (isset($a[0], $a[1])) { \\MirPsalmHost::$instance->nodeTypes[$a[0]] = $a[1]; }',
                ]
            );
            $this->nodeTypeProvider = new $providerClass();

            $sourceClass = MirShimGenerator::implement(
                'Psalm\\StatementsSource',
                'MirStatementsSourceShim',
                [
                    'getnodetypeprovider' => 'return \\MirPsalmHost::$instance->nodeTypeProvider;',
                    'getfilepath' => 'return \\MirPsalmHost::$instance->currentFile;',
                    'getrootfilepath' => 'return \\MirPsalmHost::$instance->currentFile;',
                    'getfilename' => 'return basename(\\MirPsalmHost::$instance->currentFile);',
                    'getrootfilename' => 'return basename(\\MirPsalmHost::$instance->currentFile);',
                    'getaliases' => 'return \\MirPsalmHost::$instance->currentAliases ?? new \\Psalm\\Aliases();',
                    'getsuppressedissues' => 'return [];',
                    'getsource' => 'return $this;',
                    'getfqcln' => 'return \\MirPsalmHost::$instance->callingClass;',
                    'getclassname' => 'return \\MirPsalmHost::$instance->callingClass;',
                    'getcodebase' => 'return \\MirPsalmHost::$instance->psalmCodebase() ?? throw new \\BadMethodCallException("no Psalm codebase available");',
                    'gettemplatetypemap' => 'return null;',
                    'setactivephpversion' => 'return;',
                ]
            );
            $this->statementsSource = new $sourceClass();
        }
        return $this->statementsSource;
    }

    // -- Psalm environment ------------------------------------------------------

    /**
     * A real Psalm Codebase over an empty project: enough for the type
     * comparator, location and issue machinery plugins call into, without
     * scanning anything. Class storage is empty, so class-typed comparisons
     * only succeed on identical names.
     */
    public function psalmCodebase(): ?\Psalm\Codebase
    {
        if (!class_exists(\Psalm\Internal\Analyzer\ProjectAnalyzer::class)) {
            return null;
        }
        if ($this->projectAnalyzer === null) {
            $dir = sys_get_temp_dir() . '/mir-psalm-host-' . getmypid();
            if (!is_dir($dir)) {
                mkdir($dir, 0777, true);
            }
            $config = \Psalm\Config::loadFromXML(
                $dir,
                '<?xml version="1.0"?><psalm xmlns="https://getpsalm.org/schema/config">'
                . '<projectFiles><directory name="."/></projectFiles></psalm>'
            );
            $providers = new \Psalm\Internal\Provider\Providers(new \Psalm\Internal\Provider\FileProvider());
            if ($this->composerLoader !== null && method_exists($config, 'setComposerClassLoader')) {
                $config->setComposerClassLoader($this->composerLoader);
            }
            $this->projectAnalyzer = new \Psalm\Internal\Analyzer\ProjectAnalyzer($config, $providers);
            $this->bootCodebase($config, $this->projectAnalyzer->getCodebase());
        }
        $this->registerProjectFile($this->currentFile);
        $codebase = $this->projectAnalyzer->getCodebase();
        // Psalm reads the file to compute issue locations; give it something for paths with no file on disk.
        if (!is_file($this->currentFile) && isset($codebase->file_provider)
            && method_exists($codebase->file_provider, 'addTemporaryFileChanges')) {
            $codebase->file_provider->addTemporaryFileChanges(
                $this->currentFile,
                str_repeat("\n", $this->currentSpanEnd + 1)
            );
        }
        return $codebase;
    }

    /**
     * Load Psalm's internal stubs so class-typed comparisons see PHP's own
     * classes, track references plugins create, and route class-level events
     * Psalm's scanner and populator dispatch to the registered handlers.
     */
    private function bootCodebase(object $config, \Psalm\Codebase $codebase): void
    {
        try {
            $config->visitStubFiles($codebase);
            $codebase->scanFiles();
        } catch (Throwable $e) {
            $this->warnings[] = "psalm stubs unavailable: {$e->getMessage()}";
        }
        $codebase->collect_references = true;
        $codebase->classlikes->collect_references = true;
        if (property_exists($config, 'eventDispatcher')) {
            foreach ($this->nativeHandlers as $handler) {
                $config->eventDispatcher->registerClass($handler);
            }
        }
    }

    /** Psalm only reports issues in files it knows as project files; the set is private. */
    private function registerProjectFile(string $path): void
    {
        if (isset($this->knownFiles[$path])) {
            return;
        }
        $this->knownFiles[$path] = $path;
        try {
            (new ReflectionProperty(\Psalm\Internal\Analyzer\ProjectAnalyzer::class, 'project_files'))
                ->setValue($this->projectAnalyzer, $this->knownFiles);
        } catch (ReflectionException $e) {
            $this->warnOnce('cannot register project files with this Psalm version; plugin issues may be dropped');
        }
    }

    private function absolutePath(string $file): string
    {
        if ($file === '' || $file[0] === '/' || preg_match('#^[A-Za-z]:[\\\\/]#', $file) === 1) {
            return $file;
        }
        return $this->projectRoot . '/' . $file;
    }

    private function fileContents(string $path): string
    {
        return $this->fileContents[$path] ??= (is_file($path) ? (string)file_get_contents($path) : '');
    }

    private function rawLocation(int $start, int $end): \Psalm\CodeLocation\Raw
    {
        return new \Psalm\CodeLocation\Raw(
            $this->fileContents($this->currentFile),
            $this->currentFile,
            basename($this->currentFile),
            $start,
            max($start, $end - 1) // Psalm's end offset is inclusive
        );
    }

    /** Shift PhpParser file positions by $delta so they index into the real file. */
    private static function shiftPositions(array $nodes, int $delta): void
    {
        if (!class_exists(\PhpParser\NodeTraverser::class)) {
            return;
        }
        $traverser = new \PhpParser\NodeTraverser();
        $traverser->addVisitor(new class($delta) extends \PhpParser\NodeVisitorAbstract {
            public function __construct(private int $delta)
            {
            }

            public function enterNode(\PhpParser\Node $node)
            {
                foreach (['startFilePos', 'endFilePos'] as $key) {
                    if ($node->hasAttribute($key)) {
                        $node->setAttribute($key, $node->getAttribute($key) + $this->delta);
                    }
                }
                return null;
            }
        });
        $traverser->traverse($nodes);
    }

    /**
     * Drain issues plugins raised through Psalm's IssueBuffer.
     *
     * @return list<array<string, mixed>>
     */
    private function takeIssues(): array
    {
        $issues = [];
        if (!class_exists(\Psalm\IssueBuffer::class)) {
            return $issues;
        }
        foreach (\Psalm\IssueBuffer::clear() as $perFile) {
            foreach ($perFile as $data) {
                $issues[] = [
                    'name' => $data->type,
                    'message' => $data->message,
                    'severity' => $data->severity,
                    'spanStart' => $data->from,
                    'spanEnd' => $data->to,
                ];
            }
        }
        return $issues;
    }

    // -- AfterFunctionLikeAnalysis -------------------------------------------------

    private function afterFunctionLike(array $params): array
    {
        if ($this->afterFunctionLikeHandlers === []) {
            return ['issues' => []];
        }
        $this->currentFile = $this->absolutePath((string)($params['file'] ?? 'unknown.php'));
        $this->currentSpanEnd = (int)($params['spanEnd'] ?? 0);
        $this->callingClass = isset($params['class']) ? (string)$params['class'] : null;
        $name = (string)($params['name'] ?? '');
        $spanStart = (int)($params['spanStart'] ?? 0);
        $spanEnd = (int)($params['spanEnd'] ?? $spanStart);
        $isMethod = $this->callingClass !== null;

        $codebase = $this->psalmCodebase();
        $source = $this->statementsSource();
        $location = $this->rawLocation($spanStart, $spanEnd);

        $storage = $isMethod ? new \Psalm\Storage\MethodStorage() : new \Psalm\Storage\FunctionStorage();
        $storage->cased_name = $name;
        $storage->location = $location;
        $storage->params = array_map(fn($p) => $this->buildParamStorage($p), (array)($params['params'] ?? []));

        $stmt = $this->functionLikeNode($params, $name, $isMethod, $spanStart, $spanEnd);

        $event = $this->buildEvent(
            'Psalm\\Plugin\\EventHandler\\Event\\AfterFunctionLikeAnalysisEvent',
            array_diff_key($params, ['snippet' => true]),
            [
                'stmt' => $stmt,
                'functionlike_storage' => $storage,
                'statements_source' => $source,
                'codebase' => $codebase,
                'file_replacements' => [],
                'node_type_provider' => $this->nodeTypeProvider,
            ]
        );

        foreach ($this->afterFunctionLikeHandlers as $class) {
            try {
                $class::afterStatementAnalysis($event);
            } catch (Throwable $e) {
                $this->warnOnce("after-function-like handler $class failed for $name: {$e->getMessage()}");
            }
        }
        return ['issues' => $this->takeIssues()];
    }

    // -- AfterClassLikeAnalysis ----------------------------------------------------

    private function afterClassLike(array $params): array
    {
        $empty = ['issues' => [], 'suppressedIssues' => [], 'usedClasses' => [], 'usedMethods' => []];
        if ($this->classLikeHandlers === []) {
            return $empty;
        }
        $fqcn = (string)($params['fqcn'] ?? '');
        $this->currentFile = $this->absolutePath((string)($params['file'] ?? ''));
        $this->currentSpanEnd = (int)($params['spanEnd'] ?? 0);
        $codebase = $this->psalmCodebase();
        if ($codebase === null || $fqcn === '') {
            return $empty;
        }
        $file = $this->absolutePath((string)($params['file'] ?? ''));

        try {
            $this->scanFile($codebase, $file);
            $storage = $codebase->classlike_storage_provider->get($fqcn);
        } catch (Throwable $e) {
            $this->warnOnce("cannot load $fqcn into the psalm codebase: {$e->getMessage()}");
            \Psalm\IssueBuffer::clear();
            return $empty;
        }
        $node = $this->classLikeNode($file, $fqcn);
        if ($node === null) {
            return $empty;
        }
        try {
            $this->dispatchCodebasePopulated($codebase);
        } catch (Throwable $e) {
            $this->warnOnce("after-codebase-populated handler failed: {$e->getMessage()}");
        }

        $this->currentAliases = $storage->aliases;
        $before = $this->referenceSnapshot($codebase);
        \Psalm\IssueBuffer::clear();

        $event = $this->buildEvent(
            'Psalm\\Plugin\\EventHandler\\Event\\AfterClassLikeAnalysisEvent',
            ['file' => $file, 'class' => $fqcn],
            [
                'stmt' => $node,
                'classlike_storage' => $storage,
                'codebase' => $codebase,
                'file_replacements' => [],
            ]
        );
        try {
            $codebase->config->eventDispatcher->dispatchAfterClassLikeAnalysis($event);
        } catch (Throwable $e) {
            $this->warnOnce("class-like handler failed for $fqcn: {$e->getMessage()}");
        }

        $used = $this->referenceDiff($codebase, $before, $file);
        return [
            'issues' => $this->takeIssues(),
            'suppressedIssues' => array_values(array_unique(array_map('strval', $storage->suppressed_issues))),
            'usedClasses' => $used['classes'],
            'usedMethods' => $used['methods'],
        ];
    }

    private function scanFile(\Psalm\Codebase $codebase, string $file): void
    {
        if (isset($this->scannedFiles[$file])) {
            return;
        }
        $this->scannedFiles[$file] = true;
        $this->codebaseChanged = true;
        $codebase->scanner->addFileToDeepScan($file);
        $codebase->scanFiles();
    }

    /** Psalm's ProjectAnalyzer, not the populator, announces a populated codebase. */
    private function dispatchCodebasePopulated(\Psalm\Codebase $codebase): void
    {
        if (!$this->codebaseChanged) {
            return;
        }
        $this->codebaseChanged = false;
        $eventClass = \Psalm\Plugin\EventHandler\Event\AfterCodebasePopulatedEvent::class;
        if (class_exists($eventClass)) {
            $codebase->config->eventDispatcher->dispatchAfterCodebasePopulated(new $eventClass($codebase));
        }
    }

    /** @return array<int, \PhpParser\Node\Stmt> */
    private function parsedFile(string $file): array
    {
        if (!isset($this->parsedFiles[$file])) {
            $stmts = $this->parser()->parse($this->fileContents($file)) ?? [];
            $traverser = new \PhpParser\NodeTraverser();
            $traverser->addVisitor(new \PhpParser\NodeVisitor\NameResolver());
            $this->parsedFiles[$file] = $traverser->traverse($stmts);
        }
        return $this->parsedFiles[$file];
    }

    private function classLikeNode(string $file, string $fqcn): ?\PhpParser\Node\Stmt\ClassLike
    {
        $wanted = strtolower($fqcn);
        $found = (new \PhpParser\NodeFinder())->findFirst(
            $this->parsedFile($file),
            static fn($n) => $n instanceof \PhpParser\Node\Stmt\ClassLike
                && isset($n->namespacedName)
                && strtolower($n->namespacedName->toString()) === $wanted
        );
        return $found instanceof \PhpParser\Node\Stmt\ClassLike ? $found : null;
    }

    /** @return array{classes: array<string, array<string, bool>>, members: array<string, array<string, bool>>} */
    private function referenceSnapshot(\Psalm\Codebase $codebase): array
    {
        $provider = $codebase->file_reference_provider;
        return [
            'classes' => $provider->getAllNonMethodReferencesToClasses(),
            'members' => $provider->getAllMethodReferencesToClassMembers(),
        ];
    }

    /** References a handler created since $before, with canonical class casing. */
    private function referenceDiff(\Psalm\Codebase $codebase, array $before, string $file): array
    {
        $after = $this->referenceSnapshot($codebase);
        $classes = [];
        foreach ($after['classes'] as $lc => $files) {
            if (isset($files[$file]) && !isset($before['classes'][$lc][$file])) {
                $classes[] = $this->casedClass($codebase, (string)$lc);
            }
        }
        $methods = [];
        foreach ($after['members'] as $member => $callers) {
            if (count($callers) > count($before['members'][$member] ?? [])) {
                [$class, $method] = array_pad(explode('::', (string)$member, 2), 2, '');
                if ($method !== '' && $this->declaresMethod($codebase, $class, $method)) {
                    $methods[] = $this->casedClass($codebase, $class) . '::' . $method;
                }
            }
        }
        return ['classes' => array_values(array_unique($classes)), 'methods' => array_values(array_unique($methods))];
    }

    /** Psalm also records references against every ancestor; keep only the declaring class. */
    private function declaresMethod(\Psalm\Codebase $codebase, string $class, string $method): bool
    {
        try {
            return isset($codebase->classlike_storage_provider->get($class)->methods[$method]);
        } catch (Throwable $e) {
            return false;
        }
    }

    private function casedClass(\Psalm\Codebase $codebase, string $lowercase): string
    {
        try {
            return $codebase->classlike_storage_provider->get($lowercase)->name;
        } catch (Throwable $e) {
            return $lowercase;
        }
    }

    /** Scan classes a type mentions so Psalm's comparator can look them up. */
    private function ensureTypeClasses(?\Psalm\Type\Union $type): void
    {
        $codebase = $type === null ? null : $this->psalmCodebase();
        if ($codebase === null || !method_exists($codebase, 'queueClassLikeForScanning')) {
            return;
        }
        $queued = false;
        $walk = static function ($node) use (&$walk, $codebase, &$queued): void {
            if ($node instanceof \Psalm\Type\Atomic\TNamedObject) {
                $codebase->queueClassLikeForScanning($node->value);
                $queued = true;
            }
            if (method_exists($node, 'getChildNodes')) {
                foreach ($node->getChildNodes() as $child) {
                    $walk($child);
                }
            }
        };
        $walk($type);
        if ($queued) {
            $this->codebaseChanged = true;
            try {
                $codebase->scanFiles();
            } catch (Throwable $e) {
                $this->warnOnce("cannot scan classes of a type: {$e->getMessage()}");
            }
        }
    }

    private function buildParamStorage(array $p): \Psalm\Storage\FunctionLikeParameter
    {
        $type = null;
        if (isset($p['declaredType'])) {
            try {
                $type = \Psalm\Type::parseString((string)$p['declaredType']);
            } catch (Throwable $e) {
                $type = null;
            }
        }
        $this->ensureTypeClasses($type);
        $param = new \Psalm\Storage\FunctionLikeParameter((string)$p['name'], false, $type, $type);
        foreach ((array)($p['attributes'] ?? []) as $a) {
            $attrLocation = $this->rawLocation((int)$a['spanStart'], (int)$a['spanEnd']);
            $args = [];
            foreach ((array)($a['args'] ?? []) as $arg) {
                try {
                    $argType = isset($arg['type']) ? \Psalm\Type::parseString((string)$arg['type']) : \Psalm\Type::getMixed();
                } catch (Throwable $e) {
                    $argType = \Psalm\Type::getMixed();
                }
                $this->ensureTypeClasses($argType);
                $args[] = new \Psalm\Storage\AttributeArg($arg['name'] ?? null, $argType, $attrLocation);
            }
            $param->attributes[] = new \Psalm\Storage\AttributeStorage(
                (string)$a['class'],
                $args,
                $attrLocation,
                $attrLocation
            );
        }
        return $param;
    }

    /** Re-parse the declaration so plugins can walk real PhpParser nodes at real file offsets. */
    private function functionLikeNode(array $params, string $name, bool $isMethod, int $spanStart, int $spanEnd): \PhpParser\Node\FunctionLike
    {
        $snippet = isset($params['snippet']) ? (string)$params['snippet'] : null;
        if ($snippet !== null && $isMethod) {
            try {
                $prefix = '<?php class MirWrap { ';
                $stmts = $this->parser()->parse($prefix . $snippet . ' }');
                $node = (new \PhpParser\NodeFinder())->findFirstInstanceOf($stmts ?? [], \PhpParser\Node\Stmt\ClassMethod::class);
                if ($node !== null) {
                    self::shiftPositions([$node], $spanStart - strlen($prefix));
                    return $node;
                }
            } catch (Throwable $e) {
                // fall through to a bare node
            }
        }
        $node = $isMethod
            ? new \PhpParser\Node\Stmt\ClassMethod($name)
            : new \PhpParser\Node\Stmt\Function_($name);
        $node->setAttribute('startFilePos', $spanStart);
        $node->setAttribute('endFilePos', max($spanStart, $spanEnd - 1));
        return $node;
    }

    private array $seenWarnings = [];

    private function warnOnce(string $message): void
    {
        if (isset($this->seenWarnings[$message])) {
            return;
        }
        $this->seenWarnings[$message] = true;
        fwrite(STDERR, "mir psalm bridge: $message\n");
    }
}

// -- Main loop ----------------------------------------------------------------

$host = new MirPsalmHost();
$stdout = fopen('php://stdout', 'w');

while (($line = fgets(STDIN)) !== false) {
    $line = trim($line);
    if ($line === '') {
        continue;
    }
    $request = json_decode($line, true);
    if (!is_array($request)) {
        continue;
    }
    $id = $request['id'] ?? 0;
    $method = (string)($request['method'] ?? '');

    // Swallow any direct plugin output (echo/print) so it cannot corrupt the
    // JSON protocol on stdout.
    ob_start();
    try {
        $result = $host->dispatch($method, (array)($request['params'] ?? []));
        $response = ['id' => $id, 'result' => $result];
    } catch (Throwable $e) {
        $response = ['id' => $id, 'error' => $e->getMessage()];
    }
    $stray = ob_get_clean();
    if ($stray !== '' && $stray !== false) {
        fwrite(STDERR, $stray);
    }

    fwrite($stdout, json_encode($response, JSON_UNESCAPED_SLASHES) . "\n");
    fflush($stdout);

    if ($method === 'shutdown') {
        break;
    }
}
