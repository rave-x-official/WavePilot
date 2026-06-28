import { cn } from "../../lib/utils";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export function Input({ className, label, error, id, ...props }: InputProps) {
  return (
    <div className="flex flex-col gap-1.5">
      {label && (
        <label htmlFor={id} className="text-sm font-medium text-text-secondary">
          {label}
        </label>
      )}
      <input
        id={id}
        className={cn(
          "input",
          error && "border-danger focus:ring-danger/50 focus:border-danger",
          className,
        )}
        {...props}
      />
      {error && <p className="text-xs text-danger">{error}</p>}
    </div>
  );
}
