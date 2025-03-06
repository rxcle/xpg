
namespace Results;

public readonly record struct Result<T>
{
    private readonly Error _error;
    private readonly Ok<T> _ok;

    public Result(Error error) => 
        _error = error;

    public Result(Ok<T> ok) =>
        _ok = ok;

    public bool IsOk => 
        _ok.IsSet && !_error.IsSet;

    public bool IsError =>
        _error.IsSet && !_ok.IsSet;

    public T Value => 
        IsOk ? _ok.Value : default!;

    public Ok<T> Ok => _ok;

    public Error Error => _error;

    public override string ToString() =>
        IsError ? _error.ToString() : _ok.ToString();

    public static implicit operator Result<T>(Result<Empty> result) =>
        result.IsError
            ? new(result.Error)
            : new(new Ok<T>(default!, result.Ok.IsSet));

    public static implicit operator Result<Empty>(Result<T> result) =>
        result.IsError 
            ? new(result.Error) 
            : new(new Ok<Empty>(default!, result.Ok.IsSet));

    public static implicit operator Result<T>(T value) =>
        new(new Ok<T>(value, true));

    public static implicit operator T(Result<T> result) =>
        result.IsError ? default! : result.Value;
}

public readonly record struct Ok<T>(T Value, bool IsSet = true)
{
    public override string ToString() => IsSet ? $"Ok: {Value}" : "None";
}

public readonly record struct Error(string Message, Exception? Exception = null, bool IsSet = true)
{
    public override string ToString() => IsSet ? $"Error: {Message}" : "None";
}

public readonly record struct Empty
{
    public override string ToString() => "Empty";
}

public static class Result
{
    public static Result<T> AsOk<T>(T value) => 
        new(new Ok<T>(value));

    public static Result<T> AsOk<T>() =>
        new(new Ok<T>(default!, true));

    public static Result<Empty> AsOk() =>
        new(new Ok<Empty>(default, true));

    public static Result<Empty> AsError(Error error) =>
        new(error);

    public static Result<Empty> AsError(Exception exception) =>
        new(new Error(exception?.Message ?? "Exception", exception, true));

    public static Result<T> Try<T>(Func<Result<T>> action)
    {
        try
        {
            return action();
        }
        catch (Exception ex)
        {
            return AsError(ex);
        }
    }

    public static Result<T> Try<T>(Func<T> action) => 
        Try(() => AsOk(action()));

    public static Result<Empty> Try(Action action) => 
        Try(() =>
        {
            action();
            return AsOk();
        });

    public static Result<U> Into<T, U>(this Result<T> result) =>
        result.IsError
            ? new Result<U>(result.Error)
            : new Result<U>(new Ok<U>(default!, result.Ok.IsSet));

    public static Result<U> Into<T, U>(this Result<T> result, Func<T, Result<U>> mapper) =>
        result.IsError
            ? new Result<U>(result.Error)
            : mapper(result.Value);

    public static Result<U> Into<T, U>(this Result<T> result, Func<T, U> mapper) =>
        result.IsError
            ? new Result<U>(result.Error)
            : new Result<U>(new Ok<U>(mapper(result.Value)));

    public static Result<U> Into<T, U>(this Result<T> result, Func<Result<T>, Result<U>> mapper) =>
        mapper(result);

    public static Result<U> Into<T, U>(this Result<T> result, Func<Result<T>, U> mapper) =>
        mapper(result);

    public static Result<T> Expect<T>(this Result<T> result) =>
        !result.IsError ? result : throw (result.Error.Exception ?? new Exception(result.Error.Message));

    public static Result<T> With<T>(this Result<T> result, Action<Result<T>> action)
    {
        action(result);
        return result;
    }

    public static Result<T> WithOk<T>(this Result<T> result, Action<T> action)
    {
        if (result.IsOk)
            action(result.Value);
        return result;
    }

    public static Result<T> WithError<T>(this Result<T> result, Action<Error> action)
    {
        if (result.IsError)
            action(result.Error);
        return result;
    }
}

