# Caiman Assembly Grammar

Just a quick summary of the assembly grammar, mostly for my own sanity.  Also
useful for presentation purposes and redesign, at least initially.  

# Semi-formal Grammar

Note that newlines are meaningful, for readability we use `~` to indicate a
newline. Except when a newline is expected, whitespace is meaningless.  Keywords
are surrounded by `*` since otherwise they get lost in the amount of notation.
`...` means the preceding thing is a list, separated by the preceding character.
So `v,...` would be a list of `v` separated by commas.  Some can be empty, some
can't, but I won't bother notating the difference

## Meta Variables

### Tokens

```
id := usual variable naming
n := positive number
type_name := $id
fn_name := @id
var_name := %id | %n
funclet_loc := fn_name.var_name

type := i32 | type_name
place := *local* | *cpu* | *gpu*
stage := *unbound* | *bound* | *encoded* | *submitted* | *ready* | *dead*
constant := ni32

tag := value_tag | timeline_tag | spatial_tag
standard_tag := *none* 
    | *operation* funclet_loc
    | *input* funclet_loc
    | *output* funclet_loc

value_tag := standard_tag
    | *function_input* funclet_loc
    | *function_output* funclet_loc
    | *halt* n
timeline_tag := standard_tag
spatial_tag := standard_tag
```

### Extras

```
dict_key := id | var_name
dict_value := id | [dict_value~,...] | { dictionary } | place | stage | tag
dictionary := dict_key : dict_value~... // we don't check fields during parsing
```

## Program definition

```
program := t~f~e~p // order matters
// Specifically types, then funclets, then extras, then pipelines
```

## Types

```
t := *types*[type_def~...]
type_def := type | id type_name { dictionary }
```

## Funclets

```
f := funclet~...
funclet_def := extern | funclet
```

### External

```
external_names := *external_cpu* | *external_gpu*
extern := external_names type fn_name(type,...)
```

### Funclet

```
funclet := *value* value_funclet 
    | *schedule* schedule_funclet 
    | *timeline* timeline_funclet
funclet_header := type fn_name(type var_name,...)
value_funclet := funclet_header { value_command~... }
schedule_funclet := funclet_header { schedule_command~... }
timeline_funclet := funclet_header { timeline_command~... }
```

### Commands

```
phi_command := var_name = *phi* n
return := *return* var_name

value_command := phi_command
    | var_name := *constant* constant
    | return

schedule_command := phi_command
    | var_name = *alloc*-id-id funclet_loc
    | *do*-id funclet_loc(var_name,...) -> funclet_loc
    | return

timeline_funclet := phi_command
    | return
```

## Extras

```
e := *extras* { extra~... }
extra := fn_name { dictionary }
```

## Pipelines

```
p := pipeline~...
pipeline := *pipeline* "id" = fn_name
```