namespace SignalRServer;

using Microsoft.AspNetCore.SignalR;

public class ChatHub : Hub
{
	public record Result(bool Success, string Details);
	
	public async Task<Result> SendMessage(string user, string message)
	{
		await this.Clients.All.SendAsync("ReceiveMessage", user, message);

		return new Result(true, "You chatterbox!");
	}	
}	
