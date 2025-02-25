function LoginPage() {
  return (
    <main className="container">
      <h1>Login</h1>
      <form className="column gap-14">
        <input type="text" placeholder="Username" />
        <input type="password" placeholder="Password" />
        <button type="submit">Login</button>
      </form>
    </main>

  );
}

export default LoginPage;