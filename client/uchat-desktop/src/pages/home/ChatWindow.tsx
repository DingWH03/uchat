import React from 'react';
import { useParams } from 'react-router-dom';

const ChatWindow: React.FC = () => {
  const { userId } = useParams<{ userId: string }>();
  
  return (
    <div className="chat-window">
      <h3>与 {userId} 的聊天</h3>
      <div className="messages">
        {/* 这里可以是聊天记录 */}
        <div><b>John Doe:</b> Hi!</div>
        <div><b>{userId}:</b> Hello!</div>
      </div>
      <textarea placeholder="请输入消息..."></textarea>
      <button>发送</button>
    </div>
  );
};

export default ChatWindow;
