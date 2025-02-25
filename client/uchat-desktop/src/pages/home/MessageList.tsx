import React from 'react';
import { Link } from 'react-router-dom';

const MessageList: React.FC = () => {
  const messages = [
    { id: 1, sender: 'John Doe', content: 'Hello there!' },
    { id: 2, sender: 'Jane Smith', content: 'How are you?' },
    { id: 3, sender: 'Bob Johnson', content: 'Good morning!' },
  ];

  return (
    <div className="message-list">
      <h3>消息列表</h3>
      <ul>
        {messages.map(message => (
          <li key={message.id}>
            <Link to={`/chat/${message.sender}`}>{message.sender}: {message.content}</Link>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default MessageList;
